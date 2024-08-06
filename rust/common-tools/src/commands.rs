use crate::cli::Cli;
use anyhow::{anyhow, Result};
use common_protos::{
    common,
    runtime::{self, runtime_client::RuntimeClient},
};
use common_runtime::Value;
use std::{
    io::{stdin, Read},
    path::{Path, PathBuf},
};

/// Entry point for CLI tool, taking a parsed [Cli] object.
pub async fn exec_command(cli: Cli) -> Result<()> {
    use crate::cli::Command::*;

    match cli.command {
        Run {
            module_path,
            port,
            stdin,
        } => run(module_path, port, stdin).await,
        Serve { port } => serve(port).await,
    }
}

/// Connect to runtime listening on `port`, executing
/// module found at `module_path`.
async fn run(module_path: PathBuf, port: u16, use_stdin: bool) -> Result<()> {
    let mut runtime_client = RuntimeClient::connect(format!("http://127.0.0.1:{}", port))
        .await
        .map_err(|_| {
            anyhow!(
                "Could not connect to runtime. Is `ct serve` running on port {}?",
                port
            )
        })?;

    let module_str = module_from_path(&module_path).await?;
    let input = {
        if use_stdin {
            let mut buffer = String::new();
            stdin().read_to_string(&mut buffer)?;
            Some(buffer)
        } else {
            None
        }
    };

    let output = exec_module(&mut runtime_client, &module_str, input).await?;

    if let Some(output) = output {
        println!("{}", Value::try_from(output)?);
    }
    Ok(())
}

/// Return the module found at `path` as a [String],
/// normalizing for CLI usage.
async fn module_from_path<P: AsRef<Path>>(path: P) -> Result<String> {
    let path_ref = path.as_ref();
    let module_path = if path_ref.is_absolute() {
        path_ref.canonicalize()?
    } else {
        std::env::current_dir()?.join(path_ref).canonicalize()?
    };
    tokio::fs::read_to_string(&module_path)
        .await
        .map_err(|_| anyhow!("Could not read module at {}.", module_path.display()))
}

/// Executes `module_str` with `runtime_client`.
/// Modules currently must have an output named `"output"`,
/// and an optional input named `"input"`.
async fn exec_module(
    runtime_client: &mut RuntimeClient<tonic::transport::channel::Channel>,
    module_str: &str,
    input: Option<String>,
) -> Result<Option<common::Value>> {
    let runtime::InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(runtime::InstantiateModuleRequest {
            output_shape: [("output".into(), common::ValueKind::String.into())].into(),
            default_input: [(
                "input".into(),
                common::Value {
                    variant: Some(common::value::Variant::String("".into())),
                },
            )]
            .into(),
            module_reference: Some(
                runtime::instantiate_module_request::ModuleReference::ModuleSource(
                    common::ModuleSource {
                        target: common::Target::CommonFunctionVm.into(),
                        source_code: [(
                            "module".into(),
                            common::SourceCode {
                                content_type: common::ContentType::JavaScript.into(),
                                body: module_str.into(),
                            },
                        )]
                        .into(),
                    },
                ),
            ),
        })
        .await?
        .into_inner();

    let input_io = input.unwrap_or_default();
    let runtime::RunModuleResponse { output } = runtime_client
        .run_module(runtime::RunModuleRequest {
            instance_id,
            input: [(
                "input".into(),
                common::LabeledData {
                    value: Some(common::Value {
                        variant: Some(common::value::Variant::String(input_io)),
                    }),
                    confidentiality: "Private".into(),
                    integrity: "LowIntegrity".into(),
                },
            )]
            .into(),
        })
        .await?
        .into_inner();

    match output.get("output").cloned() {
        Some(data) => match data.value {
            Some(value) => Ok(Some(value)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

/// Starts a [common_runtime] server listening on `runtime_port`,
/// with a [common_builder] server.
async fn serve(runtime_port: u16) -> Result<()> {
    use common_builder::serve as serve_builder;
    use common_runtime::serve as serve_runtime;
    use http::Uri;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;

    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_address: Uri = format!("http://{}", builder_listener.local_addr()?).parse()?;
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    let runtime_address: SocketAddr = format!("0.0.0.0:{runtime_port}")
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid port: {}", runtime_port))?;
    let runtime_listener = TcpListener::bind(runtime_address).await?;
    let runtime_task = tokio::task::spawn(serve_runtime(runtime_listener, Some(builder_address)));

    tokio::select! {
        _ = builder_task => {},
        _ = runtime_task => {},
    }

    Ok(())
}
