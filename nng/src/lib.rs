//! A safe Rust wrapper for nanomsg-next-generation
extern crate nng_sys;

#[macro_use]
extern crate log;

#[macro_use]
mod util;

mod error;
pub use error::{Error, ErrorKind, Result};

mod socket;
pub use socket::Socket;

mod pipe;
pub use pipe::{Pipe, PipeEvent};

mod protocol;
pub use protocol::Protocol;

mod dialer;
pub use dialer::{Dialer, DialerOptions};

mod listener;
pub use listener::{Listener, ListenerOptions};

mod addr;
pub use addr::SocketAddr;

mod message;
pub use message::{Message, Header, Body};

mod aio;
pub use aio::Aio;

mod ctx;
pub use ctx::Context;

pub mod options;

