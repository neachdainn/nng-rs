use std::{
	cmp::{Eq, Ordering, PartialEq, PartialOrd},
	ffi::CString,
	fmt,
	hash::{Hash, Hasher},
	os::raw::{c_int, c_void},
	panic::{catch_unwind, RefUnwindSafe},
	ptr,
	sync::{Arc, Mutex},
};

use crate::{
	aio::Aio,
	error::{Error, Result, SendResult},
	message::Message,
	pipe::{Pipe, PipeEvent},
	protocol::Protocol,
	util::validate_ptr,
};
use log::error;

type PipeNotifyFn = dyn Fn(Pipe, PipeEvent) + RefUnwindSafe + Send + Sync + 'static;

/// A nanomsg-next-generation socket.
///
/// All communication between application and remote Scalability Protocol peers
/// is done through sockets. A given socket can have multiple dialers,
/// listeners, and pipes, and may be connected to multiple transports at the
/// same time. However, a given socket will have exactly one protocol
/// associated with it and is responsible for any state machines or other
/// application-specific logic.
///
/// See the [nng documenatation][1] for more information.
///
/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_socket.5.html
#[derive(Clone, Debug)]
pub struct Socket
{
	/// The shared reference to the underlying nng socket.
	inner: Arc<Inner>,

	/// Whether or not this socket should block on sending and receiving
	nonblocking: bool,
}
impl Socket
{
	/// Creates a new socket which uses the specified protocol.
	pub fn new(t: Protocol) -> Result<Socket>
	{
		// Create the uninitialized nng_socket
		let mut socket = nng_sys::nng_socket::NNG_SOCKET_INITIALIZER;

		// Try to open a socket of the specified type
		let rv = unsafe {
			match t {
				Protocol::Bus0 => nng_sys::nng_bus0_open(&mut socket as *mut _),
				Protocol::Pair0 => nng_sys::nng_pair0_open(&mut socket as *mut _),
				Protocol::Pair1 => nng_sys::nng_pair1_open(&mut socket as *mut _),
				Protocol::Pub0 => nng_sys::nng_pub0_open(&mut socket as *mut _),
				Protocol::Pull0 => nng_sys::nng_pull0_open(&mut socket as *mut _),
				Protocol::Push0 => nng_sys::nng_push0_open(&mut socket as *mut _),
				Protocol::Rep0 => nng_sys::nng_rep0_open(&mut socket as *mut _),
				Protocol::Req0 => nng_sys::nng_req0_open(&mut socket as *mut _),
				Protocol::Respondent0 => nng_sys::nng_respondent0_open(&mut socket as *mut _),
				Protocol::Sub0 => nng_sys::nng_sub0_open(&mut socket as *mut _),
				Protocol::Surveyor0 => nng_sys::nng_surveyor0_open(&mut socket as *mut _),
			}
		};

		rv2res!(rv, Socket {
			inner:       Arc::new(Inner { handle: socket, pipe_notify: Mutex::new(None) }),
			nonblocking: false,
		})
	}

	/// Initiates a remote connection to a listener.
	///
	/// When the connection is closed, the underlying `Dialer` will attempt to
	/// re-establish the connection. It will also periodically retry a
	/// connection automatically if an attempt to connect asynchronously fails.
	///
	/// Normally, the first attempt to connect to the address indicated by the
	/// provided _url_ is done synchronously, including any necessary name
	/// resolution. As a result, a failure, such as if the connection is
	/// refused, will be returned immediately and no further action will be
	/// taken.
	///
	/// However, if the socket is set to `nonblocking`, then the connection
	/// attempt is made asynchronously.
	///
	/// Furthermore, if the connection was closed for a synchronously dialed
	/// connection, the dialer will still attempt to redial asynchronously.
	///
	/// Because the dialer is started immediately, it is generally not possible
	/// to apply extra configuration. If that is needed, or if one wishes to
	/// close the dialer before the socket, applications should consider using
	/// the `Dialer` type directly.
	///
	/// See the [nng documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_dial.3.html
	pub fn dial(&self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_dial(self.inner.handle, addr.as_ptr(), ptr::null_mut(), flags as c_int)
		};

		rv2res!(rv)
	}

	/// Initiates and starts a listener on the specified address.
	///
	/// Listeners are used to accept connections initiated by remote dialers.
	/// Unlike a dialer, listeners generally can have many connections open
	/// concurrently.
	///
	/// Normally, the act of "binding" to the address indicated by _url_ is
	/// done synchronously, including any necessary name resolution. As a
	/// result, a failure, such as if the address is already in use, will be
	/// returned immediately. However, if the socket is set to `nonblocking`
	/// then this is done asynchronously; furthermore any failure to bind will
	/// be periodically reattempted in the background.
	///
	/// Because the listener is started immediately, it is generally not
	/// possible to apply extra configuration. If that is needed, or if one
	/// wishes to close the dialer before the socket, applications should
	/// consider using the `Listener` type directly.
	///
	/// See the [nng documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_listen.3.html
	pub fn listen(&self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_listen(self.inner.handle, addr.as_ptr(), ptr::null_mut(), flags as c_int)
		};

		rv2res!(rv)
	}

	/// Sets whether or not this socket should use nonblocking operations.
	///
	/// If the socket is set to nonblocking mode, then the send and receive
	/// functions return immediately even if there are no messages available or
	/// the message cannot be sent. Otherwise, the functions will wailt until
	/// the operation can complete or any configured timer expires.
	///
	/// The default is blocking operations. This setting is _not_ propagated to
	/// other handles cloned from this one.
	pub fn set_nonblocking(&mut self, nonblocking: bool) { self.nonblocking = nonblocking; }

	/// Receives a message from the socket.
	///
	/// The semantics of what receiving a message means vary from protocol to
	/// protocol, so examination of the protocol documentation is encouraged.
	/// For example, with a _req_ socket a message may only be received after a
	/// request has been sent. Furthermore, some protocols may not support
	/// receiving data at all, such as _pub_.
	pub fn recv(&self) -> Result<Message>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe { nng_sys::nng_recvmsg(self.inner.handle, &mut msgp as _, flags as c_int) };

		let msgp = validate_ptr(rv, msgp)?;
		Ok(Message::from_ptr(msgp))
	}

	/// Sends a message on the socket.
	///
	/// The semantics of what sending a message means vary from protocol to
	/// protocol, so examination of the protocol documentation is encouraged.
	/// For example, with a _pub_ socket the data is broadcast so that any
	/// peers who have a suitable subscription will be able to receive it.
	/// Furthermore, some protocols may not support sending data (such as
	/// _sub_) or may require other conditions. For example, _rep_sockets
	/// cannot normally send data, which are responses to requests, until they
	/// have first received a request.
	///
	/// If the message cannot be sent, then it is returned to the caller as a
	/// part of the `Error`.
	pub fn send<M: Into<Message>>(&self, msg: M) -> SendResult<()>
	{
		let msg = msg.into();

		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		unsafe {
			let msgp = msg.into_ptr();
			let rv = nng_sys::nng_sendmsg(self.inner.handle, msgp.as_ptr(), flags as c_int);

			if rv != 0 {
				Err((Message::from_ptr(msgp), Error::from(rv as u32)))
			}
			else {
				Ok(())
			}
		}
	}

	/// Receive a message using the socket asynchronously.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress that is _not_ a receive operation, this function
	/// will return `Error::TryAgain`.
	pub fn recv_async(&self, aio: &Aio) -> Result<()> { aio.recv_socket(self) }

	/// Send a message using the socket asynchronously.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress, this function will return `Error::TryAgain`
	/// and return the message to the caller.
	pub fn send_async<M: Into<Message>>(&self, aio: &Aio, msg: M) -> SendResult<()>
	{
		let msg = msg.into();
		aio.send_socket(self, msg)
	}

	/// Register a callback function to be called whenever a pipe event occurs
	/// on the socket.
	///
	/// Only a single callback function can be supplied at a time. Registering a
	/// new callback implicitly unregisters any previously registered. If an
	/// error is returned, then the callback could have been registered for a
	/// subset of the events.
	///
	/// ## Panicking
	///
	/// If the callback function panics, the program will log the panic if
	/// possible and then abort. Future Rustc versions will likely do the
	/// same for uncaught panics at FFI boundaries, so this library will
	/// produce the abort in order to keep things consistent. As such, the user
	/// is responsible for either having a callback that never panics or
	/// catching and handling the panic within the callback.
	pub fn pipe_notify<F>(&self, callback: F) -> Result<()>
	where
		F: Fn(Pipe, PipeEvent),
		F: RefUnwindSafe + Send + Sync + 'static,
	{
		// Place the new callback into the inner portion.
		{
			let mut l = self.inner.pipe_notify.lock().unwrap();
			*l = Some(Arc::new(callback));
		}

		// Because we're going to override the stored closure, we absolutely need to try
		// and set the callback function for every single event. We cannot return
		// early or we risk nng trying to call into a closure that has been freed.
		let events = [
			nng_sys::nng_pipe_ev::NNG_PIPE_EV_ADD_PRE,
			nng_sys::nng_pipe_ev::NNG_PIPE_EV_ADD_POST,
			nng_sys::nng_pipe_ev::NNG_PIPE_EV_REM_POST,
		];

		// It is fine to pass in the pointer to the inner bits because the inner bits will
		// not be freed until after both the socket is no longer creating pipes and there
		// is no thread inside of the pipe notify callback.
		events
			.iter()
			.map(|&ev| unsafe {
				nng_sys::nng_pipe_notify(
					self.inner.handle,
					ev as i32,
					Some(Self::trampoline),
					&*self.inner as *const _ as _,
				)
			})
			.map(|rv| rv2res!(rv))
			.fold(Ok(()), std::result::Result::and)
	}

	/// Close the underlying socket.
	///
	/// Messages that have been submitted for sending may be flushed or
	/// delivered depending on the transport and the linger option. Further
	/// attempts to use the socket (via this handle or any other) after this
	/// call returns will result in an error. Threads waiting for operations on
	/// the socket when this call is executed may also return with an error.
	///
	/// Closing the socket while data is in transmission will likely lead to
	/// loss of that data. There is no automatic linger or flush to ensure that
	/// the socket send buffers have completely transmitted. It is recommended
	/// to wait a brief period after sending data before calling this function.
	///
	/// This function will be called automatically when all handles have been
	/// dropped.
	pub fn close(&self) { self.inner.close() }

	/// Returns the underlying `nng_socket`.
	pub(crate) fn handle(&self) -> nng_sys::nng_socket { self.inner.handle }

	/// Trampoline function for calling the pipe event closure from C.
	///
	/// This is unsafe because you have to be absolutely positive that you
	/// really do have a pointer to an `Inner` type.
	extern "C" fn trampoline(pipe: nng_sys::nng_pipe, ev: i32, arg: *mut c_void)
	{
		let res = catch_unwind(|| unsafe {
			let pipe = Pipe::from_nng_sys(pipe);
			let ev = PipeEvent::from_code(ev);

			assert!(!arg.is_null(), "Null pointer passed as argument to trampoline");
			let inner = &*(arg as *const _ as *const Inner);
			let callback = {
				// Don't hold the lock during the callback, just long enough to increment
				// the Arc's counter.
				let l = inner.pipe_notify.lock().unwrap();
				match l.as_ref() {
					Some(c) => Arc::clone(c),
					None => return,
				}
			};

			(*callback)(pipe, ev)
		});

		// See #6 for a "discussion" about why we abort.
		if let Err(e) = res {
			if let Some(s) = e.downcast_ref::<String>() {
				error!("Panic in AIO callback function: {}", s);
			}
			else if let Some(s) = e.downcast_ref::<&str>() {
				error!("Panic in AIO callback function: {}", s);
			}
			else {
				error!("Panic in AIO callback function.");
			}

			std::process::abort();
		}
	}
}

#[cfg(feature = "ffi-module")]
impl Socket
{
	/// Returns the handle to the underlying `nng_socket` object.
	pub fn nng_socket(&self) -> nng_sys::nng_socket
	{
		self.inner.handle
	}
}

impl PartialEq for Socket
{
	fn eq(&self, other: &Socket) -> bool
	{
		unsafe {
			nng_sys::nng_socket_id(self.inner.handle) == nng_sys::nng_socket_id(other.inner.handle)
		}
	}
}

impl Eq for Socket {}

impl PartialOrd for Socket
{
	fn partial_cmp(&self, other: &Socket) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Socket
{
	fn cmp(&self, other: &Socket) -> Ordering
	{
		unsafe {
			let us = nng_sys::nng_socket_id(self.inner.handle);
			let them = nng_sys::nng_socket_id(other.inner.handle);
			us.cmp(&them)
		}
	}
}

impl Hash for Socket
{
	fn hash<H: Hasher>(&self, state: &mut H)
	{
		let id = unsafe { nng_sys::nng_socket_id(self.inner.handle) };
		id.hash(state)
	}
}

#[rustfmt::skip]
expose_options!{
	Socket :: inner.handle -> nng_sys::nng_socket;

	GETOPT_BOOL = nng_sys::nng_getopt_bool;
	GETOPT_INT = nng_sys::nng_getopt_int;
	GETOPT_MS = nng_sys::nng_getopt_ms;
	GETOPT_SIZE = nng_sys::nng_getopt_size;
	GETOPT_SOCKADDR = crate::util::fake_opt;
	GETOPT_STRING = nng_sys::nng_getopt_string;
	GETOPT_UINT64 = nng_sys::nng_getopt_uint64;

	SETOPT = nng_sys::nng_setopt;
	SETOPT_BOOL = nng_sys::nng_setopt_bool;
	SETOPT_INT = nng_sys::nng_setopt_int;
	SETOPT_MS = nng_sys::nng_setopt_ms;
	SETOPT_PTR = nng_sys::nng_setopt_ptr;
	SETOPT_SIZE = nng_sys::nng_setopt_size;
	SETOPT_STRING = nng_sys::nng_setopt_string;

	Gets -> [Raw, MaxTtl, RecvBufferSize,
	         RecvTimeout, SendBufferSize,
	         SendTimeout, SocketName,
	         protocol::pair::Polyamorous,
	         protocol::reqrep::ResendTime,
	         protocol::survey::SurveyTime];
	Sets -> [ReconnectMinTime, ReconnectMaxTime,
	         RecvBufferSize, RecvMaxSize,
	         RecvTimeout, SendBufferSize,
	         SendTimeout, SocketName, MaxTtl,
	         protocol::pair::Polyamorous,
	         protocol::reqrep::ResendTime,
	         protocol::pubsub::Subscribe,
	         protocol::pubsub::Unsubscribe,
	         protocol::survey::SurveyTime,
	         transport::tcp::NoDelay,
	         transport::tcp::KeepAlive,
	         transport::tls::CaFile,
	         transport::tls::CertKeyFile,
	         transport::websocket::RequestHeaders,
	         transport::websocket::ResponseHeaders];
}

#[cfg(unix)]
mod unix_impls
{
	use super::*;
	use crate::options::{RecvFd, SendFd, SetOpt};

	impl SetOpt<RecvFd> for Socket {}
	impl SetOpt<SendFd> for Socket {}
}

/// A wrapper type around the underlying `nng_socket`.
///
/// This allows us to have mutliple Rust socket types that won't clone the C
/// socket type before Rust is done with it.
struct Inner
{
	/// Handle to the underlying nng socket.
	handle: nng_sys::nng_socket,

	/// The current pipe event callback.
	pipe_notify: Mutex<Option<Arc<PipeNotifyFn>>>,
}
impl Inner
{
	fn close(&self)
	{
		// Closing a socket should only ever return success or ECLOSED and both
		// of those mean we have nothing to drop. However, just to be sane
		// about it all, we'll warn the user if we see something odd. If that
		// ever happens, hopefully it will make its way to a bug report.
		let rv = unsafe { nng_sys::nng_close(self.handle) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED as i32,
			"Unexpected error code while closing socket ({})",
			rv
		);
	}
}

impl fmt::Debug for Inner
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		f.debug_struct("Inner")
			.field("handle", &self.handle)
			.field("pipe_notify", &self.pipe_notify.lock().unwrap().is_some())
			.finish()
	}
}

impl Drop for Inner
{
	fn drop(&mut self) { self.close() }
}
