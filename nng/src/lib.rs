//! A safe Rust wrapper for nanomsg-next-generation
#![deny(clippy::all)]

#[macro_use]
mod util;
mod addr;
mod aio;
mod ctx;
mod dialer;
mod error;
mod listener;
mod message;
mod pipe;
mod protocol;
mod socket;

pub mod options;

pub use crate::addr::SocketAddr;
pub use crate::aio::Aio;
pub use crate::ctx::Context;
pub use crate::dialer::{Dialer, DialerOptions};
pub use crate::error::{Error, ErrorKind, Result};
pub use crate::listener::{Listener, ListenerOptions};
pub use crate::message::{Body, Header, Message};
pub use crate::pipe::{Pipe, PipeEvent};
pub use crate::protocol::Protocol;
pub use crate::socket::Socket;
