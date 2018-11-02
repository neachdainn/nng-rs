//! Types for dealing with Nng contexts and asynchronous IO operations.
//!
//! Most applications will interact with _nng_ synchronously; that is that
//! functions such as `Scoket::send` will block the calling thread until the
//! operation has completed.
//!
//! Asynchronous operations behave differently. These operations are initiated
//! by the calling thread, but control returns immediately to the calling
//! thread. When the operation is subsequently completed (regardless of whether
//! this was successful or not), then a user supplied function ("callback") is
//! executed. An Aio object is associated with each asynchronous operation.
//!
//! Contexts allow the independent and concurrent use of stateful operations
//! using the same socket. For example, two different contexts created on a
//! _rep_ socket can each receive requests, and send replies to them, without
//! any regard to or interference with each other.
use crate::error::{Error, Result};
use crate::message::Message;

mod aio;
pub use self::aio::Aio;

mod ctx;
pub use self::ctx::Context;

/// Represents the state of an AIo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State
{
	/// No operation currently running.
	Inactive,

	/// A sleep operation in in progress.
	Sleeping,

	/// A send operation is either currently running or has been completed.
	Sending,

	/// A receive operation is either currently running or has been completed.
	Receiving,
}

/// Specialized result type for context wait operations.
#[must_use]
pub enum WaitResult
{
	/// The send operation was successful.
	SendOk,

	/// The send operation failed.
	SendErr(Message, Error),

	/// The receive operation was successful.
	RecvOk(Message),

	/// The receive operation failed.
	RecvErr(Error),

	/// There was no operation to wait on.
	Inactive,
}
impl WaitResult
{
	/// Retrieves the message from the result, if there is one.
	pub fn msg(self) -> Option<Message>
	{
		use self::WaitResult::*;

		match self {
			SendErr(m, _) | RecvOk(m) => Some(m),
			SendOk | RecvErr(_) | Inactive => None,
		}
	}

	/// Retrieves the error from the result, if applicable.
	pub fn err(self) -> Result<()>
	{
		use self::WaitResult::*;

		match self {
			SendErr(_, e) | RecvErr(e) => Err(e),
			SendOk | RecvOk(_) | Inactive => Ok(()),
		}
	}
}
