#![warn(missing_docs)]
//! Shared protobuf definitions for Common Modules.

/// Maximum size of gRPC messages used in server.
pub static MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;

/// Protobufs for shared Common Modules.
#[allow(missing_docs)]
pub mod common {
    tonic::include_proto!("common");

    impl From<&common_wit::Target> for Target {
        fn from(target: &common_wit::Target) -> Self {
            match target {
                common_wit::Target::CommonFunction => Target::CommonFunction,
                common_wit::Target::CommonFunctionVm => Target::CommonFunctionVm,
            }
        }
    }

    impl From<Target> for common_wit::Target {
        fn from(value: Target) -> Self {
            match value {
                Target::CommonFunction => common_wit::Target::CommonFunction,
                Target::CommonFunctionVm => common_wit::Target::CommonFunctionVm,
            }
        }
    }
}

/// Protobufs for the module builder.
#[cfg(any(feature = "builder", feature = "runtime"))]
#[allow(missing_docs)]
pub mod builder {
    tonic::include_proto!("builder");
}

/// Protobufs for the module runtime.
#[cfg(feature = "runtime")]
#[allow(missing_docs)]
pub mod runtime {
    tonic::include_proto!("runtime");
}
