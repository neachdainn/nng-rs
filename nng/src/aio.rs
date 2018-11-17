use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::panic::{catch_unwind, RefUnwindSafe};
use std::os::raw::{c_int, c_void};
use std::ptr;
use crate::error::{ErrorKind, Result, SendResult};
use crate::message::Message;
use crate::ctx::Context;
use crate::socket::Socket;

/// A handle of asynchronous I/O operations.
///
/// The handle is initialized with a completion callback which will be executed
/// when an associated asynchronous operation finishes. The callback must not
/// perform any blocking operations and must complete its execution quickly. If
/// the callback does block, this can lead ultimately to an apparent "hang" or
/// deadlock in the application.
///
/// It is possible to wait synchronously for an otherwise asynchronous
/// operation by using the function `Aio::wait`. In that case, it is
/// permissible for there to be no callback function.
pub struct Aio
{
	/// The inner `nng_aio` bits shared by Aio objects.
	///
	/// The mutex doesn't really make it all thread safe. Many of the functions
	/// relating to `nng_aio` are simply copying values into and out of the
	/// struct. However, we can (hopefully) trust nng to behave correctly and
	/// so we only need to keep the Rust side of things sane.
	///
	/// In particular, we need to make sure that only one thread is attempting
	/// to manage the message stored in the `nng_aio`. We do this by only
	/// allowing send/receive operations if we are in the correct state and by
	/// locking the state behind a mutex.
	inner: SharedInner,

	/// The box containing the callback closure, if applicable.
	///
	/// The Aio is _not_ clone so we know there there are only ever two
	/// versions of the inner data: the one the user has and the one inside the
	/// callback closure. The one in the closure does not own the closure, so
	/// we don't need to worry about cycles. If this version is dropped, the
	/// closure is also dropped, which means that version is dropped also.
	///
	/// There is no way to drop the closure without dropping the inner aio
	/// stuff first.
	///
	/// Keep in mind that the closure has technically been sent to an nng
	/// thread and it is not `Sync`. Touching it in any way is going to lead to
	/// issues.
	callback: Option<uncallable::UncallableFn>,
}
impl Aio
{
	/// Create a new asynchronous I/O handle.
	///
	/// Without a callback, the result of the I/O operation can only be
	/// retrieved after a call to `Aio::wait`.
	pub fn new() -> Result<Aio>
	{
		Ok(Aio { inner: Inner::new()? , callback: None })
	}

	/// Create a new asynchronous I/O handle.
	///
	/// The provided callback will be called on every single I/O event and
	/// `Aio::result` can be used to determine the result of the operation.
	/// With a callback provided, using `Aio::wait` is generally recommended
	/// against.
	pub fn with_callback<F>(callback: F) -> Result<Aio>
		where F: FnMut(&Aio) + Send + RefUnwindSafe + 'static
	{
		let (inner, box_cb) = Inner::with_callback(callback)?;
		Ok(Aio { inner, callback: Some(uncallable::UncallableFn::new(box_cb)) })
	}

	/// Cancel the currently running I/O operation.
	pub fn cancel(&self)
	{
		unsafe {
			let l = self.inner.lock().unwrap();
			nng_sys::nng_aio_cancel(*l.aio);
		}
	}

	/// Returns the message stored in the asynchronous I/O handle.
	///
	/// This method will only return a message in the case of a successful
	/// receive operation or a failed send operation.
	pub fn get_msg(&self) -> Option<Message>
	{
		let mut l = self.inner.lock().unwrap();

		if let State::Inactive(ref mut m) = l.state {
			m.take()
		} else { None }
	}

	/// Returns the result of the previous ansynchronous I/O operation.
	///
	/// This method will only return a result if there is currently no I/O
	/// operation running. To prevent that from being the case, call this from
	/// the callback function or after calling `Aio::wait`.
	pub fn result(&self) -> Option<Result<()>>
	{
		let l = self.inner.lock().unwrap();

		match l.state {
			State::Sending | State::Receiving | State::Sleeping => None,
			State::Inactive(_) => unsafe { Some(rv2res!(nng_sys::nng_aio_result(*l.aio))) },
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
			let l = self.inner.lock().unwrap();
			nng_sys::nng_aio_set_timeout(*l.aio, ms);
		}
	}

	/// Waits for an I/O operation to complete.
	///
	/// If there is not currently active operation, this will return
	/// immediately. If there is an active operation and a callback was
	/// defined, then the callback will run before this function returns.
	///
	/// If this method is called from within the callback function, it will
	/// return immediately.
	pub fn wait(&self)
	{
		// Nng does not define what happens if you try to wait from within the
		// callback and Garrett says that he didn't remember if it was an
		// immediate return or a deadlock. We'll just define it to be an
		// immediate return.
		if self.callback.is_none() && Arc::strong_count(&self.inner) > 1 {
			return;
		}

		// It will 100% lead to deadlocks if we acquire the lock and then wait.
		// It's a little sketchy, but we're going to pull the pointer out of
		// the lock before we wait.
		let ptr = {
			// Extra scope to be absolutely sure that the lock is dropped.
			*self.inner.lock().unwrap().aio
		};

		// Do the actual wait
		unsafe { nng_sys::nng_aio_wait(ptr) }

		// Now, if there was no callback function, we need to manually take
		// care of the state. Since we made it this far, we know that we aren't
		// _in_ the callback function, so we check need to check if we own the
		// box.
		if self.callback.is_none() {
			self.event_update_state();
		}
	}

	/// Send a message using the context asynchronously.
	///
	/// The API of the asynchronous I/O stuff needs to match what _nng_ does,
	/// but this type has a lot of bookkeeping associated with sending and
	/// receiving messages that I do not want to expose to users. As such, the
	/// `Context::send` is really just a wrapper around this function.
	pub(crate) fn send_ctx(&self, ctx: &Context, msg: Message) -> SendResult<()>
	{
		let mut l = self.inner.lock().unwrap();

		if let State::Inactive(_) = l.state {
			unsafe {
				nng_sys::nng_aio_set_msg(*l.aio, msg.into_ptr());
				nng_sys::nng_ctx_send(ctx.handle(), *l.aio);

				l.state = State::Sending;

				Ok(())
			}
		} else { Err((msg, ErrorKind::TryAgain.into())) }
	}

	/// Send a message using the socket asynchronously.
	///
	/// The API of the asynchronous I/O stuff needs to match what _nng_ does,
	/// but this type has a lot of bookkeeping associated with sending and
	/// receiving messages that I do not want to expose to users. As such, the
	/// `Socket::send` is really just a wrapper around this function.
	pub(crate) fn send_socket(&self, socket: &Socket, msg: Message) -> SendResult<()>
	{
		let mut l = self.inner.lock().unwrap();

		if let State::Inactive(_) = l.state {
			unsafe {
				nng_sys::nng_aio_set_msg(*l.aio, msg.into_ptr());
				nng_sys::nng_send_aio(socket.handle(), *l.aio);

				l.state = State::Sending;

				Ok(())
			}
		} else { Err((msg, ErrorKind::TryAgain.into())) }
	}

	/// Receive a message using the context asynchronously.
	///
	/// The API of the asynchronous I/O should match what _nng_ does, but the
	/// Aio object has a lot of bookkeeping. So the real meat of the operation
	/// happens in this function but it is exposed to the user as a context
	/// method.
	pub(crate) fn recv_ctx(&self, ctx: &Context) -> Result<()>
	{
		let mut l = self.inner.lock().unwrap();

		match l.state {
			State::Inactive(_) | State::Receiving => unsafe {
				nng_sys::nng_ctx_recv(ctx.handle(), *l.aio);
				l.state = State::Receiving;

				Ok(())
			},
			_ => Err(ErrorKind::TryAgain.into()),
		}
	}

	/// Receive a message using the socket asynchronously.
	///
	/// The API of the asynchronous I/O should match what _nng_ does, but the
	/// Aio object has a lot of bookkeeping. So the real meat of the operation
	/// happens in this function but it is exposed to the user as a socket
	/// method.
	pub(crate) fn recv_socket(&self, socket: &Socket) -> Result<()>
	{
		let mut l = self.inner.lock().unwrap();

		match l.state {
			State::Inactive(_) | State::Receiving => unsafe {
				nng_sys::nng_recv_aio(socket.handle(), *l.aio);
				l.state = State::Receiving;

				Ok(())
			},
			_ => Err(ErrorKind::TryAgain.into()),
		}
	}

	/// Performs and asynchronous sleep operation.
	///
	/// If the sleep finishes completely, it will never return an error. If a
	/// timeout has been set and it is shorter than the duration of the sleep
	/// operation, the sleep operation will end early with
	/// `ErrorKind::TimedOut`.
	///
	/// The result of this operation will be available either after calling
	/// `Aio::wait` or inside of the callback function. If the send operation
	/// fails, the message can be retrieved using the `Aio::get_msg` function.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress, this function will return `ErrorKind::TryAgain`.
	pub fn sleep(&self, dur: Duration) -> Result<()>
	{
		let ms = crate::duration_to_nng(Some(dur));

		let mut l = self.inner.lock().unwrap();

		if let State::Inactive(_) = l.state {
			unsafe { nng_sys::nng_sleep_aio(ms, *l.aio) }

			l.state = State::Sleeping;

			Ok(())
		} else { Err(ErrorKind::TryAgain.into()) }
	}

	/// Update the state of the Aio.
	///
	/// This function should only be called after a wait (non-callback version)
	/// or at the very start of the trampoline closure (callback version).
	fn event_update_state(&self)
	{
		assert!(
			self.callback.is_none() || Arc::strong_count(&self.inner) == 1,
			"Trying to update state on incorrect Aio instance"
		);

		let mut l = self.inner.lock().unwrap();

		let mut old_state = State::Inactive(None);
		std::mem::swap(&mut l.state, &mut old_state);

		l.state = match old_state {
			State::Inactive(m) => State::Inactive(m),
			State::Sleeping => State::Inactive(None),
			State::Sending => unsafe {
				// If there was an error, we need to extract the message.
				let rv = nng_sys::nng_aio_result(*l.aio);
				let msg = if rv != 0 {
					Some(Message::from_ptr(nng_sys::nng_aio_get_msg(*l.aio)))
				} else { None };

				State::Inactive(msg)
			},
			State::Receiving => unsafe {
				// If there was _no_ error, we need to extract the message.
				let rv = nng_sys::nng_aio_result(*l.aio);
				let msg = if rv == 0 {
					Some(Message::from_ptr(nng_sys::nng_aio_get_msg(*l.aio)))
				} else { None };

				State::Inactive(msg)
			},
		};
	}
}

/// Type alias for a shared inner object.
type SharedInner = Arc<Mutex<Inner>>;

/// The inner workings of an Aio object.
///
/// Unfortunately, most of the `nng_aio` operations aren't actually thread
/// safe. The library assumes that the user is aware of ownership transfers and
/// does not violate them. That's perfectly reasonable for a C library but not
/// for Rust. As such, we need an inner object that we can lock and use to keep
/// track of the `nng_aio` state.
struct Inner
{
	/// The asynchronous I/O context.
	aio: AioPtr,

	/// The current state of the the I/O context.
	state: State,
}
impl Inner
{
	/// Allocates a new asynchronous I/O context without a callback.
	fn new() -> Result<SharedInner>
	{
		let mut aio = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_aio_alloc(&mut aio, None, ptr::null_mut()) };
		validate_ptr!(rv, aio);

		Ok(Arc::new(Mutex::new(Inner { aio: AioPtr(aio), state: State::Inactive(None) })))
	}

	/// Allocates a new asynchronous I/O context with a callback.
	fn with_callback<F>(mut callback: F) -> Result<(SharedInner, Box<FnMut() + Send + RefUnwindSafe + 'static>)>
		where F: FnMut(&Aio) + Send + RefUnwindSafe + 'static
	{
		// We start by creating an (unallocated) shared inner object. The
		// object is uninitialized but it has a fixed address now so we can
		// allocate the `nng_aio`. The thing to watch out for is making sure
		// that we don't try to drop this until it is fully initialized.
		let shared_inner = Arc::new(Mutex::new(Inner {
			aio: AioPtr(ptr::null_mut()),
			state: State::Inactive(None),
		}));

		// Now, because we have a callback, we need to do some crazy
		// trampolining.
		let cb_aio = Aio {
			inner: shared_inner.clone(),
			callback: None,
		};

		// Within this trampoline, we also need to add a lock to prevent NNG
		// from entering the closure mutiple times simultaneously. An
		// alternative to the lock was is to require that the user provide a
		// `Fn() + Sync` closure but that really puts a damper on the
		// ergonomics of the API.
		let callback_lock = Mutex::new(move || {
			cb_aio.event_update_state();
			callback(&cb_aio)
		});
		let trampoline = move || (callback_lock.lock().unwrap_or_else(|p| p.into_inner()))();

		// We currently control every version of this mutex, so we know
		// that it is uncontested and not poisoned.
		let (rv, box_fn) = unsafe {
			let mut l = shared_inner.lock().unwrap();
			Inner::aio_alloc_trampoline(&mut *l.aio, trampoline)
		};

		/*} else {
			let mut l = shared_inner.lock().unwrap();
			let rv = unsafe { nng_sys::nng_aio_alloc(&mut *l.aio, None, ptr::null_mut()) };
			(rv, None)
		};*/

		// Normally, we would check the return code against the pointer - if
		// the pointer was null with a valid return code, we panic. If the
		// return code was non-zero, we assumed that there was no memory to be
		// freed and dropped as necessary.
		//
		// Unfortunately, we can't really do that here. We do not have the
		// ability to drop all the references to the inner Aio stuff. So, if we
		// get a non-zero return code and the pointer is not null, then we're
		// going to have to leak memory in order to prevent trying to free bad
		// stuff. Looking through `nng`, that should never, ever happen but
		// this is a place where I wan't to play it safe.
		if rv != 0 {
			if !shared_inner.lock().unwrap().aio.is_null() {
				// Leak a reference to the shared inner so that the `Inner` is
				// never dropped.
				std::mem::forget(shared_inner);
			}
			Err(ErrorKind::from_code(rv).into())
		} else {
			assert!(!shared_inner.lock().unwrap().aio.is_null(), "Nng returned null pointer from successful function");
			Ok((shared_inner, box_fn))
		}
	}

	/// Utility function for allocating an `nng_aio`.
	///
	/// We need this because we need to be able to get the type of the closure
	/// and Rust (currently) doesn't have a way to do that.
	///
	/// We cannot provide the box to this function because it needs to be given
	/// the raw closure in order to get the closure's type.
	unsafe fn aio_alloc_trampoline<F>(
		aio: *mut *mut nng_sys::nng_aio,
		trampoline: F,
	) -> (c_int, Box<FnMut() + Send + RefUnwindSafe + 'static>)
		where F: FnMut() + Send + RefUnwindSafe + 'static
	{
		let mut box_fn = Box::new(trampoline);
		let rv = nng_sys::nng_aio_alloc(aio, Some(Inner::trampoline::<F>), &mut *box_fn as *mut _ as _);
		(rv, box_fn)
	}

	/// Trampoline function for calling a closure from C.
	///
	/// This is unsafe because you have to be absolutely positive that `T` is
	/// really actually truly the type of the closure.
	extern "C" fn trampoline<F>(arg: *mut c_void)
		where F: FnMut() + RefUnwindSafe + Send + 'static
	{
		// TODO: I don't like just logging the error. Somehow, this panic
		// should make its way back to the user. See issue #6.
		let res = catch_unwind(|| unsafe {
			let callback_ptr = arg as *mut F;
			if callback_ptr.is_null() {
				// This should never, ever happen.
				panic!("Null argument given to trampoline function");
			}

			(*callback_ptr)()
		});

		if let Err(e) = res {
			error!("Panic in callback function: {:?}", e);
		}
	}
}

impl Drop for Inner
{
	fn drop(&mut self)
	{
		// There are some error paths that lead to the pointer being null.
		if !self.aio.is_null() {
			unsafe { nng_sys::nng_aio_free(*self.aio) }
		}
	}
}

/// A newtype in order to make `nng_aio` pointers `Send`.
#[repr(transparent)]
struct AioPtr(*mut nng_sys::nng_aio);
impl std::ops::Deref for AioPtr
{
	type Target = *mut nng_sys::nng_aio;

	fn deref(&self) -> &Self::Target
	{
		&self.0
	}
}
impl std::ops::DerefMut for AioPtr
{
	fn deref_mut(&mut self) -> &mut Self::Target
	{
		&mut self.0
	}
}
unsafe impl Send for AioPtr {}

/// Represents the state of an Aio.
enum State
{
	/// No operation currently running.
	Inactive(Option<Message>),

	/// A sleep operation is in progress.
	Sleeping,

	/// A send operation is currently running.
	Sending,

	/// A receive operation is currently running.
	Receiving,
}

mod uncallable
{
	use super::*;

	/// A newtype to prevent calling the boxed function.
	pub struct UncallableFn
	{
		_func: Box<FnMut() + Send + RefUnwindSafe + 'static>,
	}
	impl UncallableFn
	{
		/// Creates a new wrapper around the boxed function.
		pub fn new(func: Box<FnMut() + Send + RefUnwindSafe + 'static>) -> Self
		{
			UncallableFn { _func: func }
		}
	}
}
