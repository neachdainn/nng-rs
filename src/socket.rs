use std::{
	cmp::{Eq, Ordering, PartialEq, PartialOrd},
	convert::TryFrom,
	error,
	ffi::CString,
	fmt,
	hash::{Hash, Hasher},
	num::NonZeroU32,
	os::raw::{c_int, c_void},
	ptr,
	sync::{Arc, RwLock},
};

use crate::{
	aio::Aio,
	error::{Error, Result, SendResult},
	message::Message,
	pipe::{Pipe, PipeEvent},
	protocol::Protocol,
	util::{abort_unwind, validate_ptr},
};

type PipeNotifyFn = dyn Fn(Pipe, PipeEvent) + Send + Sync + 'static;

/// An NNG socket.
///
/// All communication between application and remote Scalability Protocol peers
/// is done through sockets. A given socket can have multiple dialers,
/// listeners, and pipes, and may be connected to multiple transports at the
/// same time. However, a given socket will have exactly one protocol
/// associated with it and is responsible for any state machines or other
/// application-specific logic.
///
/// See the [NNG documentation][1] for more information.
///
/// [1]: https://nanomsg.github.io/nng/man/v1.2.2/nng_socket.5.html
#[derive(Clone, Debug)]
pub struct Socket
{
	/// The shared reference to the underlying NNG socket.
	inner: Arc<Inner>,
}
impl Socket
{
	/// Creates a new socket which uses the specified protocol.
	///
	/// # Errors
	///
	/// * [`NotSupported`]: Protocol is not enabled.
	/// * [`OutOfMemory`]: Insufficient memory available.
	///
	/// [`NotSupported`]: enum.Error.html#variant.NotSupported
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
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
			inner: Arc::new(Inner { handle: socket, pipe_notify: RwLock::new(None) }),
		})
	}

	/// Initiates a remote connection to a listener.
	///
	/// When the connection is closed, the underlying `Dialer` will attempt to
	/// re-establish the connection.
	///
	/// The first attempt to connect to the address indicated by the
	/// provided _url_ is done synchronously, including any necessary name
	/// resolution. As a result, a failure, such as if the connection is
	/// refused, will be returned immediately and no further action will be
	/// taken.
	///
	/// If the connection was closed for a synchronously dialed
	/// connection, the dialer will still attempt to redial asynchronously.
	///
	/// Because the dialer is started immediately, it is generally not possible
	/// to apply extra configuration. If that is needed, or if one wishes to
	/// close the dialer before the socket, applications should consider using
	/// the `Dialer` type directly.
	///
	/// See the [NNG documentation][1] for more information.
	///
	/// # Errors
	///
	/// * [`AddressInvalid`]: An invalid _url_ was specified.
	/// * [`Closed`]: The socket is not open.
	/// * [`ConnectionRefused`]: The remote peer refused the connection.
	/// * [`ConnectionReset`]: The remote peer reset the connection.
	/// * [`DestUnreachable`]: The remote address is not reachable.
	/// * [`OutOfMemory`]: Insufficient memory is available.
	/// * [`PeerAuth`]: Authentication or authorization failure.
	/// * [`Protocol`]: A protocol error occurred.
	///
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.2.2/nng_dial.3.html
	/// [`AddressInvalid`]: enum.Error.html#variant.AddressInvalid
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`ConnectionRefused`]: enum.Error.html#variant.ConnectionRefused
	/// [`ConnectionReset`]: enum.Error.html#variant.ConnectionReset
	/// [`DestUnreachable`]: enum.Error.html#variant.DestUnreachable
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	/// [`PeerAuth`]: enum.Error.html#variant.PeerAuth
	/// [`Protocol`]: enum.Error.html#variant.Protocol
	pub fn dial(&self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let rv = unsafe { nng_sys::nng_dial(self.inner.handle, addr.as_ptr(), ptr::null_mut(), 0) };

		rv2res!(rv)
	}

	/// Initiates and starts a listener on the specified address.
	///
	/// Listeners are used to accept connections initiated by remote dialers.
	/// Unlike a dialer, listeners generally can have many connections open
	/// concurrently.
	///
	/// The act of "binding" to the address indicated by _url_ is
	/// done synchronously, including any necessary name resolution. As a
	/// result, a failure, such as if the address is already in use, will be
	/// returned immediately.
	///
	/// Because the listener is started immediately, it is generally not
	/// possible to apply extra configuration. If that is needed, or if one
	/// wishes to close the dialer before the socket, applications should
	/// consider using the `Listener` type directly.
	///
	/// See the [NNG documentation][1] for more information.
	///
	/// # Errors
	///
	/// * [`AddressInUse`]: The address specified by _url_ is already in use.
	/// * [`AddressInvalid`]: An invalid _url_ was specified.
	/// * [`Closed`]: The socket is not open.
	/// * [`OutOfMemory`]: Insufficient memory is available.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.2.2/nng_listen.3.html
	/// [`AddressInUse`]: enum.Error.html#variant.AddressInUse
	/// [`Addressinvalid`]: enum.Error.html#variant.Addressinvalid
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	pub fn listen(&self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let rv =
			unsafe { nng_sys::nng_listen(self.inner.handle, addr.as_ptr(), ptr::null_mut(), 0) };

		rv2res!(rv)
	}

	/// Asynchronously initiates a remote connection to a listener.
	///
	/// When the connection is closed, the underlying `Dialer` will attempt to
	/// re-establish the connection. It will also periodically retry a
	/// connection automatically if an attempt to connect asynchronously fails.
	///
	/// Because the dialer is started immediately, it is generally not possible
	/// to apply extra configuration. If that is needed, or if one wishes to
	/// close the dialer before the socket, applications should consider using
	/// the `Dialer` type directly.
	///
	/// See the [NNG documentation][1] for more information.
	///
	/// # Errors
	///
	/// * [`AddressInvalid`]: An invalid _url_ was specified.
	/// * [`Closed`]: The socket is not open.
	/// * [`ConnectionRefused`]: The remote peer refused the connection.
	/// * [`ConnectionReset`]: The remote peer reset the connection.
	/// * [`DestUnreachable`]: The remote address is not reachable.
	/// * [`OutOfMemory`]: Insufficient memory is available.
	/// * [`PeerAuth`]: Authentication or authorization failure.
	/// * [`Protocol`]: A protocol error occurred.
	///
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.2.2/nng_dial.3.html
	/// [`AddressInvalid`]: enum.Error.html#variant.AddressInvalid
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`ConnectionRefused`]: enum.Error.html#variant.ConnectionRefused
	/// [`ConnectionReset`]: enum.Error.html#variant.ConnectionReset
	/// [`DestUnreachable`]: enum.Error.html#variant.DestUnreachable
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	/// [`PeerAuth`]: enum.Error.html#variant.PeerAuth
	/// [`Protocol`]: enum.Error.html#variant.Protocol
	///
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.2.2/nng_dial.3.html
	pub fn dial_async(&self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let flags = nng_sys::NNG_FLAG_NONBLOCK as c_int;
		let rv =
			unsafe { nng_sys::nng_dial(self.inner.handle, addr.as_ptr(), ptr::null_mut(), flags) };

		rv2res!(rv)
	}

	#[doc(hidden)]
	#[deprecated(since = "1.0.0-rc.1", note = "This is equivalent to `Socket::listen`")]
	pub fn listen_async(&self, url: &str) -> Result<()> { self.listen(url) }

	/// Receives a message from the socket.
	///
	/// The semantics of what receiving a message means vary from protocol to
	/// protocol, so examination of the protocol documentation is encouraged.
	/// For example, with a _req_ socket a message may only be received after a
	/// request has been sent. Furthermore, some protocols may not support
	/// receiving data at all, such as _pub_.
	///
	/// # Errors
	///
	/// * [`Closed`]: The socket is not open.
	/// * [`IncorrectState`]: The socket cannot receive data in this state.
	/// * [`NotSupported`]: The protocol does not support receiving.
	/// * [`OutOfMemory`]: Insufficient memory is available.
	/// * [`TimedOut`]: The operation timed out.
	///
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`IncorrectState`]: enum.Error.html#variant.IncorrectState
	/// [`NotSupported`]: enum.Error.html#variant.NotSupported
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	/// [`TimedOut`]: enum.Error.html#variant.TimedOut
	pub fn recv(&self) -> Result<Message>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_recvmsg(self.inner.handle, &mut msgp as _, 0) };

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
	/// _sub_) or may require other conditions. For example, _rep_ sockets
	/// cannot normally send data, which are responses to requests, until they
	/// have first received a request.
	///
	/// If the message cannot be sent, then it is returned to the caller as a
	/// part of the `Error`.
	///
	/// # Errors
	///
	/// * [`Closed`]: The socket is not open.
	/// * [`IncorrectState`]: The socket cannot send messages in this state.
	/// * [`MessageTooLarge`]: The message is too large.
	/// * [`NotSupported`]: The protocol does not support sending messages.
	/// * [`OutOfMemory`]: Insufficient memory available.
	/// * [`TimedOut`]: The operation timed out.
	///
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`IncorrectState`]: enum.Error.html#variant.IncorrectState
	/// [`MessageTooLarge`]: enum.Error.html#variant.MessageTooLarge
	/// [`NotSupported`]: enum.Error.html#variant.NotSupported
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	/// [`TimedOut`]: enum.Error.html#variant.TimedOut
	pub fn send<M: Into<Message>>(&self, msg: M) -> SendResult<()>
	{
		let msg = msg.into();

		unsafe {
			let msgp = msg.into_ptr();
			let rv = nng_sys::nng_sendmsg(self.inner.handle, msgp.as_ptr(), 0);

			if let Some(e) = NonZeroU32::new(rv as u32) {
				Err((Message::from_ptr(msgp), Error::from(e)))
			}
			else {
				Ok(())
			}
		}
	}

	/// Attempts to receives a message from the socket.
	///
	/// The semantics of what receiving a message means vary from protocol to
	/// protocol, so examination of the protocol documentation is encouraged.
	/// For example, with a _req_ socket a message may only be received after a
	/// request has been sent. Furthermore, some protocols may not support
	/// receiving data at all, such as _pub_.
	///
	/// If no message is available, this function will immediately return.
	///
	/// # Errors
	///
	/// * [`Closed`]: The socket is not open.
	/// * [`IncorrectState`]: The socket cannot receive data in this state.
	/// * [`NotSupported`]: The protocol does not support receiving.
	/// * [`OutOfMemory`]: Insufficient memory is available.
	/// * [`TryAgain`]: The operation would block.
	///
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`IncorrectState`]: enum.Error.html#variant.IncorrectState
	/// [`NotSupported`]: enum.Error.html#variant.NotSupported
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	/// [`TryAgain`]: enum.Error.html#variant.TryAgain
	pub fn try_recv(&self) -> Result<Message>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let flags = nng_sys::NNG_FLAG_NONBLOCK as c_int;
		let rv = unsafe { nng_sys::nng_recvmsg(self.inner.handle, &mut msgp as _, flags) };

		let msgp = validate_ptr(rv, msgp)?;
		Ok(Message::from_ptr(msgp))
	}

	/// Attempts to sends a message on the socket.
	///
	/// The semantics of what sending a message means vary from protocol to
	/// protocol, so examination of the protocol documentation is encouraged.
	/// For example, with a _pub_ socket the data is broadcast so that any
	/// peers who have a suitable subscription will be able to receive it.
	/// Furthermore, some protocols may not support sending data (such as
	/// _sub_) or may require other conditions. For example, _rep_ sockets
	/// cannot normally send data, which are responses to requests, until they
	/// have first received a request.
	///
	/// If the message cannot be sent (e.g., there are no peers or there is
	/// backpressure from the peers) then this function will return immediately.
	/// If the message cannot be sent, then it is returned to the caller as a
	/// part of the `Error`.
	///
	/// # Errors
	///
	/// * [`Closed`]: The socket is not open.
	/// * [`IncorrectState`]: The socket cannot send messages in this state.
	/// * [`MessageTooLarge`]: The message is too large.
	/// * [`NotSupported`]: The protocol does not support sending messages.
	/// * [`OutOfMemory`]: Insufficient memory available.
	/// * [`TryAgain`]: The operation would block.
	///
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`IncorrectState`]: enum.Error.html#variant.IncorrectState
	/// [`MessageTooLarge`]: enum.Error.html#variant.MessageTooLarge
	/// [`NotSupported`]: enum.Error.html#variant.NotSupported
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	/// [`TryAgain`]: enum.Error.html#variant.TryAgain
	pub fn try_send<M: Into<Message>>(&self, msg: M) -> SendResult<()>
	{
		let msg = msg.into();
		let flags = nng_sys::NNG_FLAG_NONBLOCK as c_int;

		unsafe {
			let msgp = msg.into_ptr();
			let rv = nng_sys::nng_sendmsg(self.inner.handle, msgp.as_ptr(), flags);

			if let Some(e) = NonZeroU32::new(rv as u32) {
				Err((Message::from_ptr(msgp), Error::from(e)))
			}
			else {
				Ok(())
			}
		}
	}

	/// Start a receive operation using the given `Aio` and return immediately.
	///
	/// # Errors
	///
	/// * [`IncorrectState`]: The `Aio` already has a running operation.
	///
	/// [`IncorrectState`]: enum.Error.html#variant.IncorrectState
	pub fn recv_async(&self, aio: &Aio) -> Result<()> { aio.recv_socket(self) }

	/// Start a send operation on the given `Aio` and return immediately.
	///
	/// # Errors
	///
	/// * [`IncorrectState`]: The `Aio` already has a running operation.
	///
	/// [`IncorrectState`]: enum.Error.html#variant.IncorrectState
	pub fn send_async<M: Into<Message>>(&self, aio: &Aio, msg: M) -> SendResult<()>
	{
		let msg = msg.into();
		aio.send_socket(self, msg)
	}

	/// Register a callback function to be called whenever a pipe event occurs
	/// on the socket.
	///
	/// Only a single callback function can be supplied at a time. Registering a
	/// new callback implicitly unregisters any previously registered.
	///
	/// # Errors
	///
	/// None specified.
	///
	/// # Panics
	///
	/// If the callback function panics, the program will log the panic if
	/// possible and then abort. Future Rustc versions will likely do the
	/// same for uncaught panics at FFI boundaries, so this library will
	/// produce the abort in order to keep things consistent. As such, the user
	/// is responsible for either having a callback that never panics or
	/// catching and handling the panic within the callback.
	pub fn pipe_notify<F>(&self, callback: F) -> Result<()>
	where
		F: Fn(Pipe, PipeEvent) + Send + Sync + 'static,
	{
		// Place the new callback into the inner portion.
		{
			let mut l = self.inner.pipe_notify.write().unwrap();
			*l = Some(Box::new(callback));
		}

		// Because we're going to override the stored closure, we absolutely need to try
		// and set the callback function for every single event. We cannot return
		// early or we risk NNG trying to call into a closure that has been freed.
		let events = [
			nng_sys::NNG_PIPE_EV_ADD_PRE,
			nng_sys::NNG_PIPE_EV_ADD_POST,
			nng_sys::NNG_PIPE_EV_REM_POST,
		];

		// It is fine to pass in the pointer to the inner bits because the inner bits
		// will not be freed until after both the socket is no longer creating pipes and
		// there is no thread inside of the pipe notify callback.
		//
		// Also, at least since NNG v1.1.1, this can only fail if the socket is invalid
		// (which we're pretty dang sure isn't the case here). However, I am keeping the
		// return type as a `Result` for future-proofing reasons.
		events
			.iter()
			.map(|&ev| unsafe {
				nng_sys::nng_pipe_notify(
					self.inner.handle,
					ev,
					Some(Self::trampoline),
					&*self.inner as *const _ as _,
				)
			})
			.map(|rv| rv2res!(rv))
			.fold(Ok(()), std::result::Result::and)
	}

	#[doc(hidden)]
	#[deprecated(since = "1.0.0-rc.1", note = "Use `TryFrom` instead")]
	pub fn into_raw(self) -> Option<RawSocket> { RawSocket::try_from(self).ok() }

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
	unsafe extern "C" fn trampoline(
		pipe: nng_sys::nng_pipe,
		ev: nng_sys::nng_pipe_ev,
		arg: *mut c_void,
	)
	{
		abort_unwind(|| {
			let pipe = Pipe::from_nng_sys(pipe);
			let ev = PipeEvent::from_code(ev);

			assert!(!arg.is_null(), "Null pointer passed as argument to trampoline");
			let inner = &*(arg as *const _ as *const Inner);

			// There are three alternatives to holding this lock during the callback:
			//
			// 1. Changing the `Box` to an `Arc` and cloning it.
			// 2. Pushing all callbacks into a `Vec` and only dropping them when the socket
			//    is dropped, only needing the lock to add a new callback.
			// 3. Leak the memory, requiring no locks.
			//
			// The first option seems a little gross and requires extra atomic operations
			// which may or may not be cheap. The second seems like it might be counter
			// intuitive to the user as to when the closure is dropped. The third seems very
			// unprofessional. In contrast, doing this version will only block setting a new
			// callback (which is probably indicative of bad design).
			//
			// If people disagree, feel free to open a Gitlab issue.
			if let Some(callback) = &*inner.pipe_notify.read().unwrap() {
				(*callback)(pipe, ev)
			}
		});
	}
}

#[cfg(feature = "ffi-module")]
impl Socket
{
	/// Returns the handle to the underlying `nng_socket` object.
	pub fn nng_socket(&self) -> nng_sys::nng_socket { self.inner.handle }
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

	GETOPT_BOOL = nng_sys::nng_socket_get_bool;
	GETOPT_INT = nng_sys::nng_socket_get_int;
	GETOPT_MS = nng_sys::nng_socket_get_ms;
	GETOPT_SIZE = nng_sys::nng_socket_get_size;
	GETOPT_SOCKADDR = nng_sys::nng_socket_get_addr;
	GETOPT_STRING = nng_sys::nng_socket_get_string;
	GETOPT_UINT64 = nng_sys::nng_socket_get_uint64;

	SETOPT = nng_sys::nng_socket_set;
	SETOPT_BOOL = nng_sys::nng_socket_set_bool;
	SETOPT_INT = nng_sys::nng_socket_set_int;
	SETOPT_MS = nng_sys::nng_socket_set_ms;
	SETOPT_PTR = nng_sys::nng_socket_set_ptr;
	SETOPT_SIZE = nng_sys::nng_socket_set_size;
	SETOPT_STRING = nng_sys::nng_socket_set_string;

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
	use crate::options::{GetOpt, RecvFd, SendFd};

	impl GetOpt<RecvFd> for Socket {}
	impl GetOpt<SendFd> for Socket {}
}

/// A wrapper type around the underlying `nng_socket`.
///
/// This allows us to have mutliple Rust socket types that won't clone the C
/// socket type before Rust is done with it.
struct Inner
{
	/// Handle to the underlying NNG socket.
	handle: nng_sys::nng_socket,

	/// The current pipe event callback.
	pipe_notify: RwLock<Option<Box<PipeNotifyFn>>>,
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
			.field("pipe_notify", &self.pipe_notify.read().unwrap().is_some())
			.finish()
	}
}

impl Drop for Inner
{
	fn drop(&mut self) { self.close() }
}

/// A socket that is open in "raw" mode.
///
/// Most NNG applications will interact with sockets in "cooked" mode. This mode will automatically
/// handle the full semantics of the protocol, such as _req_ sockets automatically matching a reply
/// to a request or resenting a request periodically if no reply was received.
///
/// However, there are situations, such as with [proxies][1], where it is desirable to bypass these
/// semantics and pass messages without any extra handling. This is possible with "raw" mode
/// sockets.
///
/// When using these sockets, the user is responsible for applying any additional socket semantics
/// which typically means inspecting the message [`Header`] on incoming messages and supplying them
/// on outgoing messages.
///
/// [1]: fn.forwarder.html
/// [`Header`]: struct.Header.html
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RawSocket
{
	/// The NNG socket.
	pub socket: Socket,

	/// Make non-exhaustive.
	_priv: (),
}

impl RawSocket
{
	/// Creates a new "raw" socket of the specified protocol.
	///
	/// # Errors
	///
	/// * [`NotSupported`]: Protocol is not enabled.
	/// * [`OutOfMemory`]: Insufficient memory available.
	///
	/// [`NotSupported`]: enum.Error.html#variant.NotSupported
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	pub fn new(t: Protocol) -> Result<RawSocket>
	{
		// This code is largely copied from the Socket impl.
		let mut socket = nng_sys::nng_socket::NNG_SOCKET_INITIALIZER;
		let rv = unsafe {
			match t {
				Protocol::Bus0 => nng_sys::nng_bus0_open_raw(&mut socket as *mut _),
				Protocol::Pair0 => nng_sys::nng_pair0_open_raw(&mut socket as *mut _),
				Protocol::Pair1 => nng_sys::nng_pair1_open_raw(&mut socket as *mut _),
				Protocol::Pub0 => nng_sys::nng_pub0_open_raw(&mut socket as *mut _),
				Protocol::Pull0 => nng_sys::nng_pull0_open_raw(&mut socket as *mut _),
				Protocol::Push0 => nng_sys::nng_push0_open_raw(&mut socket as *mut _),
				Protocol::Rep0 => nng_sys::nng_rep0_open_raw(&mut socket as *mut _),
				Protocol::Req0 => nng_sys::nng_req0_open_raw(&mut socket as *mut _),
				Protocol::Respondent0 => nng_sys::nng_respondent0_open_raw(&mut socket as *mut _),
				Protocol::Sub0 => nng_sys::nng_sub0_open_raw(&mut socket as *mut _),
				Protocol::Surveyor0 => nng_sys::nng_surveyor0_open_raw(&mut socket as *mut _),
			}
		};

		if let Some(e) = NonZeroU32::new(rv as u32) {
			return Err(Error::from(e));
		}

		let socket =
			Socket { inner: Arc::new(Inner { handle: socket, pipe_notify: RwLock::new(None) }) };

		Ok(RawSocket { socket, _priv: () })
	}
}

impl TryFrom<Socket> for RawSocket
{
	type Error = CookedSocketError;

	fn try_from(socket: Socket) -> std::result::Result<Self, Self::Error>
	{
		use crate::options::{Options, Raw};

		if socket.get_opt::<Raw>().expect("Socket should have \"raw\" option available") {
			Ok(RawSocket { socket, _priv: () })
		}
		else {
			Err(CookedSocketError)
		}
	}
}

/// Indicates that the socket is not in "raw" mode.
#[derive(Debug)]
pub struct CookedSocketError;

impl fmt::Display for CookedSocketError
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "Socket is in \"cooked\" (not \"raw\") mode")
	}
}

impl error::Error for CookedSocketError
{
	fn description(&self) -> &str { "Socket is in \"cooked\" (not \"raw\") mode" }
}
