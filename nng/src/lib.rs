//! A safe Rust wrapper for nanomsg-next-generation
#![deny(clippy::all)]

#[macro_use]
mod util;
mod error;
mod socket;
mod pipe;
mod protocol;
mod dialer;
mod listener;
mod addr;
mod message;
mod aio;
mod ctx;

pub mod options;

pub use crate::error::{Error, ErrorKind, Result};
pub use crate::socket::Socket;
pub use crate::pipe::{Pipe, PipeEvent};
pub use crate::protocol::Protocol;
pub use crate::dialer::{Dialer, DialerOptions};
pub use crate::listener::{Listener, ListenerOptions};
pub use crate::addr::SocketAddr;
pub use crate::message::{Message, Header, Body};
pub use crate::aio::Aio;
pub use crate::ctx::Context;
