#![warn(missing_docs)]

//! This crate contains shared Common WIT definitions and helpers for assembling
//! them during build steps and other logistical processes.

use std::{collections::BTreeMap, str::FromStr};

use serde::{Deserialize, Serialize};

/// WIT definition for `common:function`
pub const COMMON_FUNCTION_WIT: &[u8] =
    include_bytes!("../../../wit/common/function/wit/function.wit");

/// WIT definition for `common:io`
pub const COMMON_IO_WIT: &[u8] = include_bytes!("../../../wit/common/io/wit/io.wit");

/// WIT definition for `common:data`
pub const COMMON_DATA_WIT: &[u8] = include_bytes!("../../../wit/common/data/wit/data.wit");

/// A target that some candidate source code may express the implementation of
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Target {
    /// The most basic target: a Common Module
    #[serde(rename = "common:function/module")]
    CommonFunction,
    /// Effectively the same as a Common Module, but intepreted w/o a compile step
    #[serde(rename = "common:function/virtual-module")]
    CommonFunctionVm,
}

impl Target {
    /// The presumptive WIT world that corresponds to a give [WitTarget]
    pub fn world(&self) -> &'static str {
        match self {
            Target::CommonFunction => "common:function/module",
            Target::CommonFunctionVm => "common:function/virtual-module",
        }
    }
}

impl FromStr for Target {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "common:function/module" => Target::CommonFunction,
            "common:function/virtual-module" => Target::CommonFunctionVm,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unrecognized target: {s}"),
                ));
            }
        })
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Target::CommonFunction => "common:function/module",
                Target::CommonFunctionVm => "common:function/virtual-module",
            }
        )
    }
}

/// A map of files that correspond to a give [WitTarget]

#[repr(transparent)]
#[derive(Clone)]
pub struct WitTargetFileMap(BTreeMap<String, &'static [u8]>);

impl AsRef<BTreeMap<String, &'static [u8]>> for WitTargetFileMap {
    fn as_ref(&self) -> &BTreeMap<String, &'static [u8]> {
        &self.0
    }
}

impl WitTargetFileMap {
    /// Efficiently writes the files in the [WitTargetFileMap] to a target
    /// location on the local filesystem
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn write_to(self, target_directory: &std::path::Path) -> Result<(), std::io::Error> {
        use std::io::Cursor;

        use tokio::fs::{create_dir_all, File};
        use tokio::io::copy;
        use tokio::task::JoinSet;

        let mut write_threads = JoinSet::<Result<(), std::io::Error>>::new();

        for (fragment, bytes) in self.0.into_iter() {
            let target_directory = target_directory.to_owned();

            write_threads.spawn(async move {
                let path = target_directory.join(fragment);
                let base_directory = path.parent().ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Bad path: {}", path.display()),
                ))?;

                create_dir_all(base_directory).await?;

                let mut file = File::options()
                    .read(false)
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .await?;

                copy(&mut Cursor::new(bytes), &mut file).await?;

                Ok(())
            });
        }

        while let Some(result) = write_threads.join_next().await {
            result??;
        }

        Ok(())
    }
}

impl From<Target> for WitTargetFileMap {
    fn from(value: Target) -> Self {
        WitTargetFileMap::from(&value)
    }
}

impl From<&Target> for WitTargetFileMap {
    fn from(value: &Target) -> Self {
        WitTargetFileMap(match value {
            Target::CommonFunction | Target::CommonFunctionVm => BTreeMap::from([
                ("target.wit".into(), COMMON_FUNCTION_WIT),
                ("deps/io/io.wit".into(), COMMON_IO_WIT),
                ("deps/data/data.wit".into(), COMMON_DATA_WIT),
            ]),
        })
    }
}

#[cfg(all(not(target_arch = "wasm32"), test))]
mod tests {
    use anyhow::Result;
    use tempfile::TempDir;

    use crate::{Target, WitTargetFileMap};

    #[tokio::test]
    async fn it_can_write_a_wit_hierarchy_to_the_file_system() -> Result<()> {
        let output = TempDir::new()?;
        let file_map: WitTargetFileMap = Target::CommonFunction.into();

        file_map.clone().write_to(output.path()).await?;

        for (fragment, bytes) in file_map.as_ref().iter() {
            let path = output.path().join(fragment);
            assert!(tokio::fs::try_exists(&path).await?);
            assert_eq!(tokio::fs::read(&path).await?.as_slice(), *bytes);
        }

        Ok(())
    }
}
