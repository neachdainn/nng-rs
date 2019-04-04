//! Asynchonous I/O operaions.
use crate::ctx::Context;
use crate::error::{Result, SendResult};
use crate::message::Message;
use crate::socket::Socket;

/// A structure used for asynchronous I/O operation.
pub trait Aio: self::private::Sealed { }

/// All non-public AIO related items.
pub(crate) mod private
{
	use super::*;

	/// A type used to seal the `Aio` trait to prevent users from implementing it for foreign types.
	pub trait Sealed
	{
		/// Sends the message on the provided socket.
		fn send_socket(&self, socket: &Socket, msg: Message) -> SendResult<()>;

		/// Receives a message on the provided socket.
		fn recv_socket(&self, socket: &Socket) -> Result<()>;

		/// Sends the message on the provided context.
		fn send_ctx(&self, ctx: &Context, msg: Message) -> SendResult<()>;

		/// Receives a message on the provided context.
		fn recv_ctx(&self, ctx: &Context) -> Result<()>;
	}
}
