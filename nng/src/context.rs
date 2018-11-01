//! Types for dealing with Nng contexts and asynchronous IO operations.
//!
//! Contexts allow the independent and concurrent use of stateful operations
//! using the same socket. For example, two different contexts created on a
//! _rep_ socket can each receive requests, and send replies to them, without
//! any regard to or interference with each other.
use std::ptr;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::panic::{catch_unwind, RefUnwindSafe};
use std::os::raw::{c_int, c_void};
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

/// A socket context with asynchronous callback.
///
/// This version has a callback function that is called every time an event
/// happens.
pub struct CbContext
{
	/// The shared state of all contexts.
	inner: Arc<CbContextInner>,

	/// The stored closure that is a part of the callback function.
	///
	/// Only one context needs to store this because all other versions are
	/// inside the closure. It gets really recursive and as such, we absolutely
	/// need to make sure that this is dropped after the inner bits. If the
	/// inner bits are dropped after, then the inner bits might accidentally
	/// try to go into a closure which has been freed.
	///
	/// This should never, ever, ever be called or touched at all.
	_callback: Option<Box<FnMut() + Send + RefUnwindSafe + 'static>>,
}
impl CbContext
{
	/// Creates a new context for the socket that calls the given callback
	/// function.
	pub fn new<F>(socket: &Socket, mut callback: F) -> Result<CbContext>
		where F: FnMut(&CbContext) + Send + RefUnwindSafe + 'static
	{
		// Initialize the context
		let mut ctx = nng_sys::NNG_CTX_INITIALIZER;
		let rv = unsafe {
			nng_sys::nng_ctx_open(&mut ctx as _, socket.handle())
		};
		rv2res!(rv)?;

		// Create the inner object first, since we need to reference it from
		// within the trampoline function.
		let inner = Arc::new(CbContextInner {
			mutex: Mutex::new((State::Inactive, AioPtr(ptr::null_mut()))),
			ctx,
		});

		// Create the trampoline function that holds the reference to the inner
		// portion.
		let rc = CbContext {
			inner: inner.clone(),
			_callback: None,
		};
		let trampoline = move || {
			callback(&rc);
		};

		// Now we can try and allocate the AIO object. Even though there is no
		// one competing for the lock, we're going to hold onto it this whole
		// time.
		let box_fn = {
			let mut lock = inner.mutex.lock().unwrap();
			let ptr = lock.1.get();
			let (rv, box_fn) = unsafe {
				CbContext::aio_alloc(ptr as _, trampoline)
			};

			// This is a sketchy bit... But not really. If the return code is an
			// error, everything goes out of scope and is freed correctly.
			validate_ptr!(rv, ptr, {
				// The only error we should get here is `ECLOSED`, which works just
				// as well for us. Panic if that was not the cases in order to
				// encourage a bug report.
				let close_rv = unsafe { nng_sys::nng_ctx_close(ctx) };
				assert!(close_rv == 0 || close_rv == nng_sys::NNG_ECLOSED, "Unexpected error code when closing context");
			});

			box_fn
		};

		// Everything is all set up, we're good to go, return the newly created
		// Context.
		Ok(CbContext { inner, _callback: Some(box_fn) })
	}

	/// Returns the current state of the context.
	pub fn state(&self) -> State
	{
		self.inner.mutex.lock().unwrap().0
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
	pub fn send(&self, msg: Message) -> SendResult<()>
	{
		let mut lock = self.inner.mutex.lock().unwrap();

		if lock.0 != State::Inactive {
			return Err((msg, ErrorKind::IncorrectState.into()));
		}

		unsafe {
			nng_sys::nng_aio_set_msg(*lock.1.get(), msg.into_ptr());
			nng_sys::nng_ctx_send(self.inner.ctx, *lock.1.get());
		}

		lock.0 = State::Sending;
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
	pub fn recv(&self) -> Result<()>
	{
		let mut lock = self.inner.mutex.lock().unwrap();

		if lock.0 == State::Sending {
			return Err(ErrorKind::IncorrectState.into());
		}

		unsafe {
			nng_sys::nng_ctx_recv(self.inner.ctx, *lock.1.get());
		}

		lock.0 = State::Receiving;
		Ok(())
	}

	/// Cancels the currently running operation.
	pub fn cancel(&self)
	{
		unsafe {
			let mut lock = self.inner.mutex.lock().unwrap();
			nng_sys::nng_aio_cancel(*lock.1.get());
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
	pub fn set_timeout(&self, dur: Option<Duration>)
	{
		let ms = crate::duration_to_nng(dur);

		unsafe {
			let mut lock = self.inner.mutex.lock().unwrap();
			nng_sys::nng_aio_set_timeout(*lock.1.get(), ms);
		}
	}

	/// Performs and asynchronous 

	/// Returns the positive identifier for this context.
	pub fn id(&self) -> i32
	{
		let id = unsafe { nng_sys::nng_ctx_id(self.inner.ctx) };
		assert!(id > 0, "Invalid context ID returned from valid context");

		id
	}

	/// Utility function for allocating an `nng_aio`.
	///
	/// We need this because we need to be able to get the type of the closure
	/// and Rust (currently) doesn't have a way to do that.
	unsafe fn aio_alloc<F>(aio: *mut *mut nng_sys::nng_aio, trampoline: F) -> (c_int, Box<FnMut() + Send + RefUnwindSafe + 'static>)
		where F: FnMut() + Send + RefUnwindSafe + 'static
	{
		let mut box_fn = Box::new(trampoline);
		let rv = nng_sys::nng_aio_alloc(aio, Some(CbContext::trampoline::<F>), &mut *box_fn as *mut _ as _);

		(rv, box_fn)
	}

	/// Trampoline function for calling a closure from a C callback.
	///
	/// This is unsafe because you have to be absolutely positive that `T` is
	/// really actually truly the type of the closure.
	extern "C" fn trampoline<T>(arg: *mut c_void)
		where T: FnMut() + Send + RefUnwindSafe + 'static
	{
		// TODO: We need to inform the user that something went wrong. That
		// either means propagating the panic on the main thread or emitting a
		// log message about it. Right now we're just hiding it and that's no
		// good.
		let _res = catch_unwind(|| unsafe {
			let callback_ptr = arg as *mut T;
			if callback_ptr.is_null() {
				// This should never, ever happen.
				panic!("Null argument given to trampoline function");
			}

			(*callback_ptr)()
		});
	}
}

/// The libnng components of a 
struct CbContextInner
{
	/// The elements of the context that are not thread safe.
	///
	/// Basically nothing about the AIO is thread safe. In libnng, it's all
	/// basically moving pointers around and assuming that the user is doing
	/// something sane.
	///
	/// We could probably do some funky atomic stuff with the state but we're
	/// already paying the cost of the mutex so we may as well take advantage
	/// of it's ease-of-use.
	mutex: Mutex<(State, AioPtr)>,

	/// The underlying context object.
	///
	/// This type is actually thread safe, so that's cool.
	ctx: nng_sys::nng_ctx,
}

/// A wrapper around the `*mut nng_aio` so `Send` can be implementd.
struct AioPtr(*mut nng_sys::nng_aio);
impl AioPtr
{
	/// Get the value of the pointer.
	fn get(&mut self) -> &mut *mut nng_sys::nng_aio
	{
		&mut self.0
	}
}
unsafe impl Send for AioPtr {}

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
