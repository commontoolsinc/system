#![warn(missing_docs)]
//! Shared protobuf definitions for Common Modules.

/// Maximum size of gRPC messages used in server.
pub static MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;

/// Protobufs for shared Common Modules.
#[allow(missing_docs)]
pub mod common {
    tonic::include_proto!("common");

    impl From<ContentType> for ct_common::ContentType {
        fn from(value: ContentType) -> Self {
            match value {
                ContentType::JavaScript => ct_common::ContentType::JavaScript,
            }
        }
    }

    impl From<ct_common::ContentType> for ContentType {
        fn from(value: ct_common::ContentType) -> Self {
            match value {
                ct_common::ContentType::JavaScript => ContentType::JavaScript,
            }
        }
    }

    impl From<ct_common::ModuleDefinition> for ModuleDefinition {
        fn from(value: ct_common::ModuleDefinition) -> Self {
            ModuleDefinition {
                content_type: ContentType::from(value.content_type).into(),
                source: value.source,
            }
        }
    }

    impl TryFrom<ModuleDefinition> for ct_common::ModuleDefinition {
        type Error = String;
        fn try_from(value: ModuleDefinition) -> Result<Self, Self::Error> {
            Ok(ct_common::ModuleDefinition {
                content_type: ContentType::try_from(value.content_type)
                    .map_err(|e| e.to_string())?
                    .into(),
                source: value.source,
            })
        }
    }
}

/// Protobufs for the module builder.
#[cfg(any(feature = "builder"))]
#[allow(missing_docs)]
pub mod builder {
    tonic::include_proto!("builder");
}
