use std::sync::Arc;

use crate::aio::Aio;
use crate::error::{Result, SendResult};
use crate::message::Message;
use crate::socket::Socket;

/// A socket context.
///
/// The context allows for independent and concurrent use of stateful
/// operations on a single socket. Using contexts is an excellent way to write
/// simpler concurrent applications, while retaining the benefits of the
/// protocol-specific advanced processing.
///
/// Note that not all protocols allow for the creation of contexts.
///
/// ## Examples
///
/// See the documentation of the `Aio` type for examples on how to use Socket Contexts.
#[derive(Clone)]
pub struct Context
{
	/// The inner context.
	///
	/// While the `nng_ctx` type is copy and it is thread-safe, we don't want
	/// to have to worry about closing the context too early. As such, we're
	/// going to throw the inner, actual `nng_ctx` behind and Arc.
	inner: Arc<Inner>,
}
impl Context
{
	/// Creates a new socket context.
	pub fn new(socket: &Socket) -> Result<Context>
	{
		let mut ctx = nng_sys::nng_ctx::NNG_CTX_INITIALIZER;
		let rv = unsafe { nng_sys::nng_ctx_open(&mut ctx as _, socket.handle()) };

		rv2res!(rv, Context { inner: Arc::new(Inner { ctx }) })
	}

	/// Returns the positive identifier for this context.
	pub fn id(&self) -> i32
	{
		let id = unsafe { nng_sys::nng_ctx_id(self.inner.ctx) };
		assert!(id > 0, "Invalid context ID returned from valid context");

		id
	}

	/// Send a message using the context asynchronously.
	///
	/// The result of this operation will be available either after calling
	/// `Aio::wait` or inside of the callback function.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress, this function will return `ErrorKind::TryAgain`
	/// and return the message to the caller.
	pub fn send<A: Aio>(&self, aio: &mut A, msg: Message) -> SendResult<()>
	{
		aio.send_ctx(self, msg)
	}

	/// Receive a message using the context asynchronously.
	///
	/// The result of this operation will be available either after calling
	/// `Aio::wait` or inside of the callback function.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress that is _not_ a receive operation, this function
	/// will return `ErrorKind::TryAgain`.
	pub fn recv<A: Aio>(&self, aio: &mut A) -> Result<()>
	{
		aio.recv_ctx(self)
	}

	/// Closes the context.
	///
	/// Messages that have been submitted for sending may be flushed or
	/// delivered, depending on the underlying transport and the linger option.
	/// Further attempts to use the context (with this or any other handle)
	/// will result in an error. Threads waiting for operations on the context
	/// when this call is executed may also return with an error.
	///
	/// Closing the owning socket also closes this context. Additionally, the
	/// context is closed once all handles have been dropped.
	pub fn close(self)
	{
		self.inner.close()
	}

	/// Returns the inner `nng_ctx` object.
	pub(crate) fn handle(&self) -> nng_sys::nng_ctx
	{
		self.inner.ctx
	}
}

#[rustfmt::skip]
expose_options!{
	Context :: inner.ctx -> nng_sys::nng_ctx;

	GETOPT_BOOL = nng_sys::nng_ctx_getopt_bool;
	GETOPT_INT = nng_sys::nng_ctx_getopt_int;
	GETOPT_MS = nng_sys::nng_ctx_getopt_ms;
	GETOPT_SIZE = nng_sys::nng_ctx_getopt_size;
	GETOPT_SOCKADDR = crate::util::fake_opt;
	GETOPT_STRING = crate::util::fake_opt;

	SETOPT = nng_sys::nng_ctx_setopt;
	SETOPT_BOOL = nng_sys::nng_ctx_setopt_bool;
	SETOPT_INT = nng_sys::nng_ctx_setopt_int;
	SETOPT_MS = nng_sys::nng_ctx_setopt_ms;
	SETOPT_PTR = crate::util::fake_opt;
	SETOPT_SIZE = nng_sys::nng_ctx_setopt_size;
	SETOPT_STRING = crate::util::fake_opt;

	Gets -> [protocol::reqrep::ResendTime, protocol::survey::SurveyTime];
	Sets -> [protocol::reqrep::ResendTime, protocol::survey::SurveyTime];
}

/// A wrapper around an `nng_ctx`.
struct Inner
{
	ctx: nng_sys::nng_ctx,
}
impl Inner
{
	fn close(&self)
	{
		// The only time this can error is if the socket is already closed or
		// was never open. Neither of those are an issue for us.
		let rv = unsafe { nng_sys::nng_ctx_close(self.ctx) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED as i32,
			"Unexpected error code while closing context ({})",
			rv
		);
	}
}

impl Drop for Inner
{
	fn drop(&mut self)
	{
		self.close()
	}
}
