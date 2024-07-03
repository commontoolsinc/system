#![allow(missing_docs)]

pub static MAX_MESSAGE_SIZE: usize = 32 * 1024 * 1024;

pub mod common {
    tonic::include_proto!("common");
}

pub mod builder {
    tonic::include_proto!("builder");
}
