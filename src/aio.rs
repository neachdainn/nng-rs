//! Asynchonous I/O operaions.
use std::ptr;
use std::time::Duration;

use crate::ctx::Context;
use crate::error::{Error, Result, SendResult};
use crate::message::Message;
use crate::socket::Socket;

/// A structure used for asynchronous I/O operation.
pub trait Aio: self::private::Sealed { }

/// The result of an AIO operation.
#[derive(Clone, Debug)]
#[must_use]
pub enum AioResult
{
	/// No AIO operations were in progress.
	InactiveOk,

	/// The send operation was successful.
	SendOk,

	/// The send operation failed.
	///
	/// This contains the message that was being sent.
	SendErr(Message, Error),

	/// The receive operation was successful.
	RecvOk(Message),

	/// The receive operation failed.
	RecvErr(Error),

	/// The sleep operation was successful.
	SleepOk,

	/// The sleep operation failed.
	///
	/// This is almost always because the sleep was canceled and the error will usually be
	/// `Error::Canceled`.
	SleepErr(Error),
}

impl From<AioResult> for Result<Option<Message>>
{
	fn from(aio_res: AioResult) -> Result<Option<Message>>
	{
		use AioResult::*;

		match aio_res {
			InactiveOk | SendOk | SleepOk => Ok(None),
			SendErr(_, e) | RecvErr(e) | SleepErr(e) => Err(e),
			RecvOk(m) => Ok(Some(m)),
		}
	}
}

/// An AIO type that requires the user to call a blocking `wait` function.
#[derive(Debug)]
pub struct WaitingAio
{
	/// The handle to the NNG AIO object.
	handle: *mut nng_sys::nng_aio,

	/// The current state of the AIO object.
	state: State,
}

impl WaitingAio
{
	/// Create a new asynchronous I/O handle.
	pub fn new() -> Result<Self>
	{
		let mut aio = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_aio_alloc(&mut aio, None, ptr::null_mut()) };
		validate_ptr!(rv, aio);

		Ok(Self { handle: aio, state: State::Inactive })
	}

	/// Cancel the currently running I/O operation.
	pub fn cancel(&mut self)
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		unsafe { nng_sys::nng_aio_cancel(self.handle); }
	}

	/// Set the timeout of asynchronous operations.
	///
	/// This causes a timer to be started when the operation is actually started. If the timer
	/// expires before the operation is completed, then it is aborted with `Error::TimedOut`.
	///
	/// As most operations involve some context switching, it is usually a good idea to allow a
	/// least a few tens of milliseconds before timing them out - a too small timeout might not
	/// allow the operation to properly begin before giving up!
	pub fn set_timeout(&mut self, dur: Option<Duration>)
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		let ms = crate::util::duration_to_nng(dur);
		unsafe { nng_sys::nng_aio_set_timeout(self.handle, ms); }
	}

	/// Waits for an I/O operation to complete.
	///
	/// If there is not currently active operation, this will return
	/// immediately.
	///
	/// ## Example
	///
	/// ```
	/// use nng::{Aio, Socket, Protocol};
	/// use nng::aio::WaitingAio;
	///
	/// let address = "inproc://nng/aio.rs::wait";
	/// let mut socket = Socket::new(Protocol::Rep0).unwrap();
	/// let aio = WaitingAio::new().unwrap();
	///
	/// // Asynchronously wait for a message on the socket.
	/// socket.recv_async(&aio).unwrap();
	/// #
	/// # // Cancel the receive, otherwise the test will block.
	/// # aio.cancel();
	///
	/// // Wait for the asynchronous receive to complete.
	/// let res = aio.wait();
	/// ```
	pub fn wait(&mut self) -> AioResult
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		// The wait function will return immediately if there is no AIO operation started.
		let rv = unsafe {
			nng_sys::nng_aio_wait(self.handle);
			nng_sys::nng_aio_result(self.handle) as u32
		};

		let res = match (self.state, rv) {
			(State::Inactive, _) => AioResult::InactiveOk,

			(State::Sending, 0) => AioResult::SendOk,
			(State::Sending, e) => unsafe {
				let msg = Message::from_ptr(nng_sys::nng_aio_get_msg(self.handle));
				AioResult::SendErr(msg, Error::from_code(e))
			},

			(State::Receiving, 0) => unsafe {
				let msg = Message::from_ptr(nng_sys::nng_aio_get_msg(self.handle));
				AioResult::RecvOk(msg)
			},
			(State::Receiving, e) => AioResult::RecvErr(Error::from_code(e)),

			(State::Sleeping, 0) => AioResult::SleepOk,
			(State::Sleeping, e) => AioResult::SleepErr(Error::from_code(e)),
		};

		self.state = State::Inactive;
		res
	}

	/// Performs and asynchronous sleep operation.
	///
	/// If the sleep finishes completely, it will never return an error. If a
	/// timeout has been set and it is shorter than the duration of the sleep
	/// operation, the sleep operation will end early with
	/// `Error::TimedOut`.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress, this function will return `Error::TryAgain`.
	pub fn sleep(&mut self, dur: Duration) -> Result<()>
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			let ms = crate::util::duration_to_nng(Some(dur));
			unsafe { nng_sys::nng_sleep_aio(ms, self.handle); }
			self.state = State::Sleeping;

			Ok(())
		} else {
			Err(Error::TryAgain)
		}
	}
}

impl self::private::Sealed for WaitingAio
{
	fn send_socket(&mut self, socket: &Socket, msg: Message) -> SendResult<()>
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			unsafe {
				nng_sys::nng_aio_set_msg(self.handle, msg.into_ptr());
				nng_sys::nng_send_aio(socket.handle(), self.handle);
			}

			self.state = State::Sending;
			Ok(())
		} else {
			Err((msg, Error::TryAgain))
		}
	}

	fn recv_socket(&mut self, socket: &Socket) -> Result<()>
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			unsafe { nng_sys::nng_recv_aio(socket.handle(), self.handle); }

			self.state = State::Receiving;
			Ok(())
		} else {
			Err(Error::TryAgain)
		}
	}

	fn send_ctx(&mut self, ctx: &Context, msg: Message) -> SendResult<()>
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			unsafe {
				nng_sys::nng_aio_set_msg(self.handle, msg.into_ptr());
				nng_sys::nng_ctx_send(ctx.handle(), self.handle);
			}

			self.state = State::Sending;
			Ok(())
		} else {
			Err((msg, Error::TryAgain))
		}
	}

	fn recv_ctx(&mut self, ctx: &Context) -> Result<()>
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			unsafe { nng_sys::nng_ctx_recv(ctx.handle(), self.handle); }

			self.state = State::Receiving;
			Ok(())
		} else {
			Err(Error::TryAgain)
		}
	}
}
impl Aio for WaitingAio { }

impl Drop for WaitingAio
{
	fn drop(&mut self)
	{
		debug_assert!(!self.handle.is_null(), "Null AIO pointer");

		// The AIO object may contain a message that we want to free. We could either try to do some
		// crazy logic around an immediate call to `nng_aio_free`, or we could just cancel, wait,
		// and then free.
		self.cancel();
		let _ = self.wait();

		unsafe { nng_sys::nng_aio_free(self.handle) };
	}
}

/// Represents the state of the AIO object.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State
{
	/// There is currently nothing happening on the AIO.
	Inactive,

	/// A send operation is currently in progress.
	Sending,

	/// A receive operation is currently in progress.
	Receiving,

	/// The AIO object is currently sleeping.
	Sleeping,
}

/// All non-public AIO related items.
pub(crate) mod private
{
	use super::*;

	/// A type used to seal the `Aio` trait to prevent users from implementing it for foreign types.
	///
	/// This trait manages most, if not all, of the bookkeeping for the AIO objects, which is why
	/// the functions are just the transpose of the functions on Sockets and Contexts.
	pub trait Sealed
	{
		/// Sends the message on the provided socket.
		fn send_socket(&mut self, socket: &Socket, msg: Message) -> SendResult<()>;

		/// Receives a message on the provided socket.
		fn recv_socket(&mut self, socket: &Socket) -> Result<()>;

		/// Sends the message on the provided context.
		fn send_ctx(&mut self, ctx: &Context, msg: Message) -> SendResult<()>;

		/// Receives a message on the provided context.
		fn recv_ctx(&mut self, ctx: &Context) -> Result<()>;
	}
}
