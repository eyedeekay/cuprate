//! ## P2P Transports
//!
//! This crate contains additional transport implementations for Cuprate.

pub mod i2p;

pub use i2p::{I2pTransport, I2pClientConfig, I2pServerConfig};
