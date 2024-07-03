#![warn(missing_docs)]
//! Shared protobuf definitions for Common Modules.

/// Maximum size of gRPC messages used in server.
pub static MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;

/// Protobufs for shared Common Modules.
#[allow(missing_docs)]
pub mod common {
    tonic::include_proto!("common");
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
