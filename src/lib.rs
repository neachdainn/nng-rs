//! A safe Rust wrapper for NNG
//!
//! ## What Is NNG
//!
//! From the [NNG Github Repository][1]:
//!
//! > NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less
//! library, offering a simple API to solve common recurring messaging problems, such as
//! publish/subscribe, RPC-style request/reply, or service discovery. The API frees the programmer
//! from worrying about details like connection management, retries, and other common
//! considerations, so that they can focus on the application instead of the plumbing.
//!
//! ## Nng-rs
//!
//! This crate provides a safe wrapper around the NNG library, seeking to maintain an API that is
//! similar to the original library. As such, the majority of examples available online should be
//! easy to apply to this crate.
//!
//! ### Examples
//!
//! The following example uses the [intra-process][2] transport to set up a [request][3]/[reply][4]
//! socket pair. The "client" sends a String to the "server" which responds with a nice phrase.
//!
//! ```
//! use nng::*;
//!
//! // Set up the server and listen for connections on the specified address.
//! let address = "inproc://nng/lib.rs";
//! let mut server = Socket::new(Protocol::Rep0).unwrap();
//! server.listen(address).unwrap();
//!
//! // Set up the client and connect to the specified address
//! let mut client = Socket::new(Protocol::Req0).unwrap();
//! client.dial(address).unwrap();
//!
//! // Send the request from the client to the server.
//! let request = b"Ferris"[..].into();
//! client.send(request).unwrap();
//!
//! // Receive the message on the server and send back the reply
//! let request = {
//!     let req = server.recv().unwrap();
//!     String::from_utf8(req.to_vec()).unwrap()
//! };
//! assert_eq!(request, "Ferris");
//! let reply = format!("Hello, {}!", request).as_bytes().into();
//! server.send(reply).unwrap();
//!
//! // Get the response on the client side.
//! let reply = {
//!     let rep = client.recv().unwrap();
//!     String::from_utf8(rep.to_vec()).unwrap()
//! };
//! assert_eq!(reply, "Hello, Ferris!");
//! ```
//!
//! Additional examples are in the `examples` directory.
//!
//! [1]: https://github.com/nanomsg/nng
//! [2]: https://nanomsg.github.io/nng/man/v1.1.0/nng_inproc.7
//! [3]: https://nanomsg.github.io/nng/man/v1.1.0/nng_req.7
//! [4]: https://nanomsg.github.io/nng/man/v1.1.0/nng_rep.7

#![deny(clippy::all)]
#![allow(clippy::new_ret_no_self)]

#[macro_use]
mod util;
mod addr;
mod ctx;
mod dialer;
mod error;
mod listener;
mod message;
mod pipe;
mod protocol;
mod socket;

pub mod aio;
pub mod options;

pub use crate::addr::SocketAddr;
pub use crate::ctx::Context;
pub use crate::dialer::{Dialer, DialerOptions};
pub use crate::error::{Error, Result};
pub use crate::listener::{Listener, ListenerOptions};
pub use crate::message::{Body, Header, Message};
pub use crate::pipe::{Pipe, PipeEvent};
pub use crate::protocol::Protocol;
pub use crate::socket::Socket;
