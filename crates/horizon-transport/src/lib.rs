//! gRPC transport layer — strictly decoupled from the compute core.
//!
//! This crate translates between Protobuf wire types and `horizon-api` DTOs.
//! It must never import `horizon-core`, `horizon-storage`, or `horizon-geometry`.

mod convert;
mod server;
mod spatial_compute_convert;
mod spatial_compute_server;
mod traceparent;

pub use server::{serve, TransportConfig, TransportServices};
