//! A safe Rust wrapper for nanomsg-next-generation
extern crate nng_sys;

#[macro_use]
mod macros;

mod error;
pub use error::{Error, ErrorKind, Result};

mod socket;
pub use socket::{Socket, Protocol};

pub mod dialer;
pub mod listener;

mod addr;
pub use addr::SocketAddr;

pub mod message;
pub mod options;

pub mod context;
