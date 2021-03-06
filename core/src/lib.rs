#![deny(
    missing_docs, missing_debug_implementations, missing_copy_implementations, trivial_casts,
    trivial_numeric_casts, unsafe_code, unstable_features, unused_import_braces,
    unused_qualifications, warnings
)]
#![cfg_attr(feature = "dev", allow(unstable_features))]
#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

//! Core implementation for a peer in the carina network
//!
//! # Usage
//! ``` no_run
//! extern crate carina_core;
//!
//! use carina_core::{CarinaConfigBuilder, Config};
//!
//! fn main() {
//!     let config = Config::from_str(r#"---
//!         socket: /tmp/carina.sock
//!         peers: ./example_peers.yml
//!         storage: ./block_data
//!         uri: 127.0.0.1:45001
//!         secret_key: v+rETx4VtczK/QSvl9OBfJfgVPEdjNpquVUq/8GFmWo=
//!         "#).unwrap();
//!     let carina_config_builder = CarinaConfigBuilder::new().set_config(config);
//!     carina_core::init(carina_config_builder);
//! }
//! ```
extern crate base64;
extern crate carina_core_protocol;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate futures_cpupool;
#[macro_use]
extern crate log;
extern crate sodiumoxide;
extern crate yaml_rust;

/// See the config file struct for more information
mod carina_config;
mod config;
mod event;
mod udp;

pub use config::{Config, Peer};
pub use event::Event;
pub use carina_config::{CarinaConfig, CarinaConfigBuilder};

use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Initialises the library
pub fn init(builder: CarinaConfigBuilder) -> (JoinHandle<()>, UdpSocket, Arc<Mutex<CarinaConfig>>) {
    sodiumoxide::init().unwrap();

    let carina_config = builder.build();

    let socket = UdpSocket::bind(&carina_config.config.uri).unwrap();
    info!("[THREAD_UDP] Listening on  {}", carina_config.config.uri);
    let state = Arc::new(Mutex::new(carina_config));

    let socket_udp = socket.try_clone().unwrap();
    let udp_handle = udp::start(Arc::clone(&state), socket_udp);

    (udp_handle, socket.try_clone().unwrap(), Arc::clone(&state))
}
