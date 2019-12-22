use std::{
	cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
	hash::{Hash, Hasher},
	sync::Arc,
};

use crate::{
	aio::Aio,
	error::{Result, SendResult},
	message::Message,
	socket::Socket,
};

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
/// See the documentation of the `Aio` type for examples on how to use Socket
/// Contexts.
#[derive(Clone, Debug)]
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

	/// Send a message using the context asynchronously.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress, this function will return `ErrorKind::TryAgain`
	/// and return the message to the caller.
	pub fn send<M: Into<Message>>(&self, aio: &Aio, msg: M) -> SendResult<()>
	{
		let msg = msg.into();
		aio.send_ctx(self, msg)
	}

	/// Receive a message using the context asynchronously.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress that is _not_ a receive operation, this function
	/// will return `ErrorKind::TryAgain`.
	pub fn recv(&self, aio: &Aio) -> Result<()> { aio.recv_ctx(self) }

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
	pub fn close(&self) { self.inner.close() }

	/// Returns the inner `nng_ctx` object.
	pub(crate) fn handle(&self) -> nng_sys::nng_ctx { self.inner.ctx }
}

#[cfg(feature = "ffi-module")]
impl Context
{
	/// Returns the `nng_ctx` handle for this context.
	pub fn nng_ctx(&self) -> nng_sys::nng_ctx { self.handle() }
}

impl PartialEq for Context
{
	fn eq(&self, other: &Context) -> bool
	{
		unsafe { nng_sys::nng_ctx_id(self.inner.ctx) == nng_sys::nng_ctx_id(other.inner.ctx) }
	}
}

impl Eq for Context {}

impl PartialOrd for Context
{
	fn partial_cmp(&self, other: &Context) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Context
{
	fn cmp(&self, other: &Context) -> Ordering
	{
		unsafe {
			let us = nng_sys::nng_ctx_id(self.inner.ctx);
			let them = nng_sys::nng_ctx_id(other.inner.ctx);
			us.cmp(&them)
		}
	}
}

impl Hash for Context
{
	fn hash<H: Hasher>(&self, state: &mut H)
	{
		let id = unsafe { nng_sys::nng_ctx_id(self.inner.ctx) };
		id.hash(state)
	}
}

#[rustfmt::skip]
expose_options!{
	Context :: inner.ctx -> nng_sys::nng_ctx;

	GETOPT_BOOL = nng_sys::nng_ctx_get_bool;
	GETOPT_INT = nng_sys::nng_ctx_get_int;
	GETOPT_MS = nng_sys::nng_ctx_get_ms;
	GETOPT_SIZE = nng_sys::nng_ctx_get_size;
	GETOPT_SOCKADDR = nng_sys::nng_ctx_get_addr;
	GETOPT_STRING = nng_sys::nng_ctx_get_string;
	GETOPT_UINT64 = nng_sys::nng_ctx_get_uint64;

	SETOPT = nng_sys::nng_ctx_set;
	SETOPT_BOOL = nng_sys::nng_ctx_set_bool;
	SETOPT_INT = nng_sys::nng_ctx_set_int;
	SETOPT_MS = nng_sys::nng_ctx_set_ms;
	SETOPT_PTR = nng_sys::nng_ctx_set_ptr;
	SETOPT_SIZE = nng_sys::nng_ctx_set_size;
	SETOPT_STRING = nng_sys::nng_ctx_set_string;

	Gets -> [protocol::reqrep::ResendTime, protocol::survey::SurveyTime];
	Sets -> [protocol::reqrep::ResendTime, protocol::survey::SurveyTime];
}

/// A wrapper around an `nng_ctx`.
#[derive(Debug)]
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
	fn drop(&mut self) { self.close() }
}
