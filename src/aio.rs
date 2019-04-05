//! Asynchonous I/O operaions.
use std::{ptr, fmt};
use std::panic::{AssertUnwindSafe, catch_unwind, UnwindSafe};
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::ctx::Context;
use crate::error::{Error, Result, SendResult};
use crate::message::Message;
use crate::socket::Socket;
use log::error;

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
	/// use nng::{Socket, Protocol};
	/// use nng::aio::WaitingAio;
	///
	/// let address = "inproc://nng/aio.rs::wait";
	/// let mut socket = Socket::new(Protocol::Rep0).unwrap();
	/// let mut aio = WaitingAio::new().unwrap();
	///
	/// // Asynchronously wait for a message on the socket.
	/// socket.recv_async(&mut aio).unwrap();
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

/// An AIO object that utilizes a callback function in order to respond to events.
pub struct CallbackAio
{
	/// The inner AIO bits shared by all instances of this AIO.
	inner: Arc<Mutex<Inner>>,

	/// The callback function.
	///
	/// This is an `Option` because we do not want the `CallbackAio` that is inside the callback to
	/// have any sort of ownership over the callback. If it did, then there would a circlar `Arc`
	/// reference and the AIO would never be dropped. We are never going to manually call this
	/// function, so the fact that it is an option is not an issue.
	///
	/// We can assert that is is unwind safe because we literally never call this function. I don't
	/// think we could if we wanted to, which is the entire point of the black box.
	callback: Option<AssertUnwindSafe<Arc<FnOnce() + Sync + Send>>>,
}

impl CallbackAio
{
	/// Creates a new asynchronous I/O handle.
	///
	/// The provided callback will be called on every single I/O event, successful or not. It is
	/// possible that the callback will be entered multiple times simultaneously.
	///
	/// ## Panicking
	///
	/// If the callback function panics, the program will abort. This is to match the behavior
	/// specified in Rust 1.33 where the program will abort when it panics across an `extern "C"`
	/// boundary. This library will produce the abort regardless of which version of Rustc is being
	/// used.
	///
	/// The user is responsible for either having a callback that never panics or catching and
	/// handling the panic within the callback.
	pub fn new<F>(callback: F) -> Result<Self>
		where F: Fn(&mut CallbackAio, AioResult) + Sync + Send + UnwindSafe + 'static
	{
		// The shared inner needs to have a fixed location before we can do anything else.
		let inner = Arc::new(Mutex::new(Inner {
			handle: AioPtr(ptr::null_mut()),
			state: State::Inactive,
		}));

		// Now, create the CallbackAio that will be stored within the callback itself.
		let cb_aio = CallbackAio { inner: inner.clone(), callback: None };

		// We can avoid double boxing by taking the address of a generic function. Unfortunately, we
		// have no way to get the type of a closure other than calling a generic function, so we do
		// have to call another function to actually allocate the AIO.
		let bounce = move || {
			let res = unsafe {
				// Don't hold the lock during the callback, hence the extra frame.
				let mut l = cb_aio.inner.lock().unwrap();
				let rv = nng_sys::nng_aio_result(l.handle.ptr()) as u32;

				let res = match (l.state, rv) {
					(State::Inactive, _) => AioResult::InactiveOk,

					(State::Sending, 0) => AioResult::SendOk,
					(State::Sending, e) => {
						let msg = Message::from_ptr(nng_sys::nng_aio_get_msg(l.handle.ptr()));
						AioResult::SendErr(msg, Error::from_code(e))
					},

					(State::Receiving, 0) => {
						let msg = Message::from_ptr(nng_sys::nng_aio_get_msg(l.handle.ptr()));
						AioResult::RecvOk(msg)
					},
					(State::Receiving, e) => AioResult::RecvErr(Error::from_code(e)),

					(State::Sleeping, 0) => AioResult::SleepOk,
					(State::Sleeping, e) => AioResult::SleepErr(Error::from_code(e)),
				};

				l.state = State::Inactive;
				res
			};
			let mut aio = cb_aio.secret_clone();
			callback(&mut aio, res)
		};
		let callback = Some(AssertUnwindSafe(CallbackAio::alloc_trampoline(&inner, bounce)?));
		Ok(Self { inner, callback })
	}

	/// Attempts to clone the AIO object.
	///
	/// The AIO object that is passed as an argument to the callback can never be cloned. Any other
	/// instance of the AIO object can be. All clones refer to the same underlying AIO operations.
	pub fn try_clone(&self) -> Option<Self>
	{
		// The user can never, ever clone an instance of the callback AIO object. We use the
		// uniqueness of the callback pointer to know when to safely drop items. See the `Drop`
		// implementation for more details.
		if let Some(a) = &self.callback {
			let callback = Some(AssertUnwindSafe((*a).clone()));
			Some(Self { inner: self.inner.clone(), callback })
		} else {
			None
		}
	}

	/// Cancel the currently running I/O operation.
	pub fn cancel(&mut self)
	{
		self.inner.lock().unwrap().cancel();
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
		self.inner.lock().unwrap().set_timeout(dur);
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
		self.inner.lock().unwrap().sleep(dur)
	}

	/// Clones the closure CallbackAio.
	///
	/// The users must *never* have access to this function or much of what we're assuming in the
	/// drop becomes invalid.
	fn secret_clone(&self) -> Self
	{
		assert!(self.callback.is_none(), "Secret Clone called on non-closure AIO");

		Self { inner: self.inner.clone(), callback: None }
	}

	/// Utility function for allocating an `nng_aio`.
	///
	/// We need this because, in Rustc 1.31, there is zero way to get the type of the closure other
	/// than calling a generic function.
	fn alloc_trampoline<F>(
		inner: &Arc<Mutex<Inner>>,
		bounce: F
	) -> Result<Arc<FnOnce() + Sync + Send>>
		where F: Fn() + Sync + Send + UnwindSafe + 'static
	{
		let mut boxed = Box::new(bounce);
		let mut l = inner.lock().unwrap();
		let aio: *mut *mut nng_sys::nng_aio = &mut *l.handle.ptr_ref() as _;
		let rv = unsafe { nng_sys::nng_aio_alloc(
				aio,
				Some(CallbackAio::trampoline::<F>),
				&mut *boxed as *mut _ as _
		)};

		// NNG should never touch the pointer and return a non-zero code at the same time. That
		// being said, I'm going to be a pessimist and double check. If we do encounter that case,
		// the safest thing to do is make the pointer null again so that the dropping of the inner
		// can detect that something went south.
		//
		// This might leak memory (I'm not sure, depends on what NNG did), but a small amount of
		// lost memory is better than a segfaulting Rust library.
		if rv != 0 && !l.handle.ptr().is_null() {
			error!("NNG returned a non-null pointer from a failed function");
			l.handle = AioPtr(ptr::null_mut());
		}

		let ptr = l.handle.ptr();
		validate_ptr!(rv, ptr);
		Ok(Arc::new(move || { let _ = boxed; }))
	}

	/// Trampoline function for calling a closure from C.
	///
	/// This is really unsafe because you have to be absolutely positive in that the type of the
	/// pointer is actually `F`. Because we're going through C and a `c_void`, the type system does
	/// not enforce this for us.
	extern "C" fn trampoline<F>(arg: *mut c_void)
		where F: Fn() + Sync + Send + UnwindSafe + 'static
	{
		let res = catch_unwind(|| unsafe {
			let callback_ptr = arg as *const F;
			if callback_ptr.is_null() {
				// This should never happen. It means we, Nng-rs, got something wrong in the
				// allocation code.
				panic!("Null argument given to trampoline function - please open an issue");
			}

			(*callback_ptr)()
		});

		// See #6 for "discussion" about why we abort here.
		if res.is_err() {
			// No other useful information to relay to the user.
			error!("Panic in AIO callback function.");
			std::process::abort();
		}
	}
}

impl private::Sealed for CallbackAio
{
	fn send_socket(&mut self, socket: &Socket, msg: Message) -> SendResult<()>
	{
		self.inner.lock().unwrap().send_socket(socket, msg)
	}

	fn recv_socket(&mut self, socket: &Socket) -> Result<()>
	{
		self.inner.lock().unwrap().recv_socket(socket)
	}

	fn send_ctx(&mut self, ctx: &Context, msg: Message) -> SendResult<()>
	{
		self.inner.lock().unwrap().send_ctx(ctx, msg)
	}

	fn recv_ctx(&mut self, ctx: &Context) -> Result<()>
	{
		self.inner.lock().unwrap().recv_ctx(ctx)
	}
}
impl Aio for CallbackAio { }

impl fmt::Debug for CallbackAio
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		if let Some(ref a) = self.callback {
			write!(f, "CalbackAio {{ inner: {:?}, callback: Some({:p}) }}", self.inner, a)
		} else {
			write!(f, "CalbackAio {{ inner: {:?}, callback: None }}", self.inner)
		}
	}
}

impl Drop for CallbackAio
{
	fn drop(&mut self)
	{
		// This is actually a vastly critical point in the correctness of this type. The inner data
		// won't be dropped until all of the CallbackAio objects are dropped, meaning that the
		// callback function is in the process of being shut down and may already be freed by the
		// time we get to the drop method of the Inner. This means that we can't depend on the inner
		// object to shut down the NNG AIO object and we have to do that instead.
		//
		// Therefore, if we are the unique owner of the callback closure, we need to put the AIO in
		// a state where we know the callback isn't running. I *think* the `nng_aio_free` function
		// will handle this for us but the wording of the documentation is a little confusing to me.
		// Fortunately, the documentation for `nng_aio_stop` is much clearer, will definitely do
		// what we want and will also allow us to leave the actual freeing to the Inner object.
		//
		// Of course, all of this depends on the user not being able to move a closure CallbackAio
		// out of the closure. For that, all we need to do is provide it to them as a borrow and do
		// not allow it to be cloned (by them). Fortunately, if we get this wrong, I _think_ the
		// only issues will be non-responsive AIO operations.
		if let Some(ref mut a) = self.callback {
			// We share ownership of the callback, so we might need to shut things down.
			if let Some(_) = Arc::get_mut(a) {
				// We are the only owner so we need to shut down the AIO.
				let l = self.inner.lock().unwrap();
				unsafe { nng_sys::nng_aio_stop(l.handle.ptr()) }
			}
			else {
				// Just a sanity check. We need to never take a weak reference to the callback. I
				// see no reason why we would, but I'm putting this check here just in case. If this
				// panic ever happens, it is potentially a major bug.
				assert_eq!(
					Arc::weak_count(a), 0,
					"There is a weak reference in the AIO. This is a bug - please file an issue"
				);
			}
		}
	}
}

/// The shared inner items of a `CallbackAio`.
#[derive(Debug)]
pub struct Inner
{
	/// The handle to the NNG AIO object.
	handle: AioPtr,

	/// The current state of the AIO object.
	state: State,
}

impl Inner
{
	pub fn cancel(&mut self)
	{
		debug_assert!(!self.handle.ptr().is_null(), "Null AIO pointer");

		unsafe { nng_sys::nng_aio_cancel(self.handle.ptr()); }
	}

	pub fn set_timeout(&mut self, dur: Option<Duration>)
	{
		debug_assert!(!self.handle.ptr().is_null(), "Null AIO pointer");

		let ms = crate::util::duration_to_nng(dur);
		unsafe { nng_sys::nng_aio_set_timeout(self.handle.ptr(), ms); }
	}

	pub fn sleep(&mut self, dur: Duration) -> Result<()>
	{
		debug_assert!(!self.handle.ptr().is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			let ms = crate::util::duration_to_nng(Some(dur));
			unsafe { nng_sys::nng_sleep_aio(ms, self.handle.ptr()); }
			self.state = State::Sleeping;

			Ok(())
		} else {
			Err(Error::TryAgain)
		}
	}

	fn send_socket(&mut self, socket: &Socket, msg: Message) -> SendResult<()>
	{
		debug_assert!(!self.handle.ptr().is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			unsafe {
				nng_sys::nng_aio_set_msg(self.handle.ptr(), msg.into_ptr());
				nng_sys::nng_send_aio(socket.handle(), self.handle.ptr());
			}

			self.state = State::Sending;
			Ok(())
		} else {
			Err((msg, Error::TryAgain))
		}
	}

	fn recv_socket(&mut self, socket: &Socket) -> Result<()>
	{
		debug_assert!(!self.handle.ptr().is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			unsafe { nng_sys::nng_recv_aio(socket.handle(), self.handle.ptr()); }

			self.state = State::Receiving;
			Ok(())
		} else {
			Err(Error::TryAgain)
		}
	}

	fn send_ctx(&mut self, ctx: &Context, msg: Message) -> SendResult<()>
	{
		debug_assert!(!self.handle.ptr().is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			unsafe {
				nng_sys::nng_aio_set_msg(self.handle.ptr(), msg.into_ptr());
				nng_sys::nng_ctx_send(ctx.handle(), self.handle.ptr());
			}

			self.state = State::Sending;
			Ok(())
		} else {
			Err((msg, Error::TryAgain))
		}
	}

	fn recv_ctx(&mut self, ctx: &Context) -> Result<()>
	{
		debug_assert!(!self.handle.ptr().is_null(), "Null AIO pointer");

		if self.state == State::Inactive {
			unsafe { nng_sys::nng_ctx_recv(ctx.handle(), self.handle.ptr()); }

			self.state = State::Receiving;
			Ok(())
		} else {
			Err(Error::TryAgain)
		}
	}
}

impl Drop for Inner
{
	fn drop(&mut self)
	{
		// It is possible for this to be dropping while the pointer is null. The Inner struct is
		// created before the pointer is allocated and it will be dropped with a null pointer if the
		// NNG allocation fails.
		if !self.handle.ptr().is_null() {
			// If we are being dropped, then the callback is being dropped. If the callback is being
			// dropped, then an instance of `CallbackAio` shut down the AIO. This will either run the
			// callback and clean up the Message memory or the AIO didn't have an operation running and
			// there is nothing to clean up. As such, we don't need to do anything except free the AIO.
			unsafe { nng_sys::nng_aio_free(self.handle.ptr()); }
		}
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

/// Newtype to make the `*mut nng_aio` implement `Send`.
#[repr(transparent)]
#[derive(Debug)]
struct AioPtr(*mut nng_sys::nng_aio);
impl AioPtr
{
	/// Returns the wrapped pointer.
	fn ptr(&self) -> *mut nng_sys::nng_aio
	{
		self.0
	}

	/// Returns a reference to the wrapped pointer.
	fn ptr_ref(&mut self) -> &mut *mut nng_sys::nng_aio
	{
		&mut self.0
	}
}
unsafe impl Send for AioPtr { }

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
