//! Types for dealing with Nng contexts and asynchronous IO operations.
//!
//! Contexts allow the independent and concurrent use of stateful operations
//! using the same socket. For example, two different contexts created on a
//! _rep_ socket can each receive requests, and send replies to them, without
//! any regard to or interference with each other.
use std::ptr;
use std::time::Duration;
use crate::error::{Error, ErrorKind, Result, SendResult};
use crate::socket::Socket;
use crate::message::Message;

/// A socket context.
///
/// This version does not allow a callback function so it can avoid some of the
/// overhead associated with multithreading.
pub struct Context
{
	/// The actual `nng_context`.
	ctx: nng_sys::nng_ctx,

	/// The `nng_aio` paired with the context.
	///
	/// Since this version doesn't use callbacks (e.g., our execution is never
	/// on an libnng thread) we don't need to worry about locking this one.
	aio: *mut nng_sys::nng_aio,

	/// The current state of the Context.
	///
	/// We need this in order to keep track of whether or not the `nng_aio` has
	/// a message that we need to retrieve.
	state: State
}
impl Context
{
	/// Creates a new context for the socket.
	pub fn new(socket: &Socket) -> Result<Self>
	{
		let mut ctx = nng_sys::NNG_CTX_INITIALIZER;
		let rv = unsafe {
			nng_sys::nng_ctx_open(&mut ctx as _, socket.handle())
		};
		rv2res!(rv)?;

		let mut aio: *mut nng_sys::nng_aio = ptr::null_mut();
		let rv = unsafe {
			nng_sys::nng_aio_alloc(&mut aio as _, None, ptr::null_mut())
		};

		validate_ptr!(rv, aio, {
			// The only error we should get here is `ECLOSED`, which works just
			// as well for us. Panic if that was not the cases in order to
			// encourage a bug report.
			let close_rv = unsafe { nng_sys::nng_ctx_close(ctx) };
			assert!(close_rv == 0 || close_rv == nng_sys::NNG_ECLOSED, "Unexpected error code when closing context");
		});

		Ok(Context { ctx, aio, state: State::Inactive })
	}

	/// Returns the current state of the context.
	pub fn state(&self) -> State
	{
		self.state
	}

	/// Sends a message using the context.
	///
	/// This function returns immediately. To get the result of the operation,
	/// call `Context::wait` which will block until the send operation has
	/// completed. If an error occurs, then the result of the wait will contain
	/// the recovered message.
	///
	/// If another operation is currently underway, this will fail with
	/// `ErrorKind::IncorrectState`.
	pub fn send(&mut self, msg: Message) -> SendResult<()>
	{
		if self.state != State::Inactive {
			return Err((msg, ErrorKind::IncorrectState.into()));
		}

		unsafe {
			nng_sys::nng_aio_set_msg(self.aio, msg.into_ptr());
			nng_sys::nng_ctx_send(self.ctx, self.aio);
		}

		self.state = State::Sending;
		Ok(())
	}

	/// Receives a message using the context.
	///
	/// This function returns immediately. To get the result of the operation,
	/// call `Context::wait` which will block until the receive operation has
	/// completed. If the operation is successful, then the result of the wait
	/// will contain the received message.
	///
	/// If a send operation is currently underway, this will fail with
	/// `ErrorKind::IncorrectState`.
	pub fn recv(&mut self) -> Result<()>
	{
		if self.state == State::Sending {
			return Err(ErrorKind::IncorrectState.into());
		}

		unsafe {
			nng_sys::nng_ctx_recv(self.ctx, self.aio);
		}

		self.state = State::Receiving;
		Ok(())
	}

	/// Waits for the asynchronous operation to complete.
	///
	/// If a `Sending` operation failed, then the returned `Option<Message>`
	/// contains the recovered message. If a `Receiving` operation was
	/// successful, the returned `Option<Message>` contains the received
	/// message.
	pub fn wait(&mut self) -> WaitResult
	{
		// If we're inactive, don't call into libnng - I'm not 100% sure how it
		// will behave and I would rather have full control over what's going
		// on.
		if self.state == State::Inactive {
			return WaitResult::Inactive;
		}

		// Otherwise, we record our current state for later use and then wait
		// for the AIO result.
		let old_state = self.state;

		let rv = unsafe {
			nng_sys::nng_aio_wait(self.aio);
			nng_sys::nng_aio_result(self.aio)
		};

		// Regardless of the old state, we're now inactive.
		self.state = State::Inactive;

		match (old_state, rv) {
			(State::Inactive, _) => unreachable!(),
			(State::Sending, 0) => WaitResult::SendOk,
			(State::Sending, e)  => unsafe {
				let msg = Message::from_ptr(nng_sys::nng_aio_get_msg(self.aio));
				WaitResult::SendErr(msg, ErrorKind::from_code(e).into())
			},
			(State::Receiving, 0) => unsafe {
				let msg = Message::from_ptr(nng_sys::nng_aio_get_msg(self.aio));
				WaitResult::RecvOk(msg)
			},
			(State::Receiving, e) => WaitResult::RecvErr(ErrorKind::from_code(e).into()),
		}
	}

	/// Cancels the currently running operation.
	pub fn cancel(&mut self)
	{
		unsafe {
			nng_sys::nng_aio_cancel(self.aio);
		}
	}

	/// Set the timeout of asynchronous operations.
	///
	/// This causes a timer to be started when the operation is actually
	/// started. If the timer expires before the operation is completed, then
	/// it is aborted with `ErrorKind::TimedOut`.
	///
	/// As most operations involve some context switching, it is usually a good
	/// idea to allow at least a few tens of milliseconds before timing them
	/// out — a too small timeout might not allow the operation to properly
	/// begin before giving up!
	pub fn set_timeout(&mut self, dur: Option<Duration>)
	{
		let ms = crate::duration_to_nng(dur);

		unsafe {
			nng_sys::nng_aio_set_timeout(self.aio, ms);
		}
	}

	/// Returns the positive identifier for this context.
	pub fn id(&self) -> i32
	{
		let id = unsafe { nng_sys::nng_ctx_id(self.ctx) };
		assert!(id > 0, "Invalid context ID returned from valid context");

		id
	}
}

expose_options!{
	Context :: ctx -> nng_sys::nng_ctx;

	GETOPT_BOOL = nng_sys::nng_ctx_getopt_bool;
	GETOPT_INT = nng_sys::nng_ctx_getopt_int;
	GETOPT_MS = nng_sys::nng_ctx_getopt_ms;
	GETOPT_SIZE = nng_sys::nng_ctx_getopt_size;
	GETOPT_SOCKADDR = crate::fake_opt;
	GETOPT_STRING = crate::fake_opt;

	SETOPT = nng_sys::nng_ctx_setopt;
	SETOPT_BOOL = nng_sys::nng_ctx_setopt_bool;
	SETOPT_INT = nng_sys::nng_ctx_setopt_int;
	SETOPT_MS = nng_sys::nng_ctx_setopt_ms;
	SETOPT_SIZE = nng_sys::nng_ctx_setopt_size;
	SETOPT_STRING = crate::fake_opt;

	Gets -> [protocol::reqrep::ResendTime, protocol::survey::SurveyTime];
	Sets -> [protocol::reqrep::ResendTime, protocol::survey::SurveyTime];
}

impl Drop for Context
{
	fn drop(&mut self)
	{
		unsafe {
			// Kill any pending AIO operation and extract the message, if
			// needed. Since this is the non-callback version, this should
			// return immediately (and we could call `cancel` if we want)
			nng_sys::nng_aio_stop(self.aio);

			// Extract any message so that it can drop. As with above, this
			// should return immediately
			let _ = self.wait();

			// Now we can free both the AIO and the CTX. I don't think the
			// order matters here.
			nng_sys::nng_aio_free(self.aio);
			let rv = nng_sys::nng_ctx_close(self.ctx);
			assert!(rv == 0 || rv == nng_sys::NNG_ECLOSED, "Unexpected error code when closing context ({})", rv);
		}
	}
}

/// The libnng components of a 

/// Represents the state of a Context.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State
{
	/// No operation currently running.
	Inactive,

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
