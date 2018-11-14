use std::sync::Arc;
use crate::error::{Result, SendResult};
use crate::socket::Socket;
use crate::aio::Aio;
use crate::message::Message;

/// A socket context.
///
/// The context allows for independent and concurrent use of stateful
/// operations on a single socket. Using contexts is an excellent way to write
/// simpler concurrent applications, while retaining the benefits of the
/// protocol-specific advanced processing.
///
/// Note that not all protocols allow for the creation of contexts.
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
		let mut ctx = nng_sys::NNG_CTX_INITIALIZER;
		let rv = unsafe {
			nng_sys::nng_ctx_open(&mut ctx as _, socket.handle())
		};

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
	/// `Aio::wait` or inside of the callback function. If the send operation
	/// fails, the message can be retrieved using the `Aio::get_msg` function.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress, this function will return `ErrorKind::TryAgain`
	/// and return the message to the caller.
	pub fn send(&self, aio: &Aio, msg: Message) -> SendResult<()>
	{
		aio.send_ctx(self, msg)
	}

	/// Receive a message using the context asynchronously.
	///
	/// The result of this operation will be available either after calling
	/// `Aio::wait` or inside of the callback function. If the send operation
	/// fails, the message can be retrieved using the `Aio::get_msg` function.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress that is _not_ a receive operation, this function
	/// will return `ErrorKind::TryAgain`.
	pub fn recv(&self, aio:&Aio) -> Result<()>
	{
		aio.recv_ctx(self)
	}

	/// Returns the inner `nng_ctx` object.
	pub(crate) fn handle(&self) -> nng_sys::nng_ctx
	{
		self.inner.ctx
	}
}

expose_options!{
	Context :: inner.ctx -> nng_sys::nng_ctx;

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


/// A wrapper around an `nng_ctx`.
struct Inner
{
	ctx: nng_sys::nng_ctx,
}
impl Drop for Inner
{
	fn drop(&mut self)
	{
		// The only time this can error is if the socket is already closed or
		// was never open. Neither of those are an issue for us.
		let rv = unsafe { nng_sys::nng_ctx_close(self.ctx) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED,
			"Unexpected error code while closing context ({})", rv
		);
	}
}
