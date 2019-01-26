use std::ffi::CString;
use std::ptr;
use std::sync::Arc;
use nng_sys::protocol::*;
use crate::error::{ErrorKind, Result, SendResult};
use crate::message::Message;
use crate::aio::Aio;
use crate::protocol::Protocol;

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
		let mut socket = nng_sys::NNG_SOCKET_INITIALIZER;

		// Try to open a socket of the specified type
		let rv = unsafe {
			match t {
				Protocol::Bus0 => bus0::nng_bus0_open(&mut socket as *mut _),
				Protocol::Pair0 => pair0::nng_pair0_open(&mut socket as *mut _),
				Protocol::Pair1 => pair1::nng_pair1_open(&mut socket as *mut _),
				Protocol::Pub0 => pubsub0::nng_pub0_open(&mut socket as *mut _),
				Protocol::Pull0 => pipeline0::nng_pull0_open(&mut socket as *mut _),
				Protocol::Push0 => pipeline0::nng_push0_open(&mut socket as *mut _),
				Protocol::Rep0 => reqrep0::nng_rep0_open(&mut socket as *mut _),
				Protocol::Req0 => reqrep0::nng_req0_open(&mut socket as *mut _),
				Protocol::Respondent0 => survey0::nng_respondent0_open(&mut socket as *mut _),
				Protocol::Sub0 => pubsub0::nng_sub0_open(&mut socket as *mut _),
				Protocol::Surveyor0 => survey0::nng_surveyor0_open(&mut socket as *mut _),
			}
		};

		rv2res!(rv, Socket { inner: Arc::new(Inner { handle: socket }), nonblocking: false })
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
	pub fn dial(&mut self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| ErrorKind::AddressInvalid)?;
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_dial(self.inner.handle, addr.as_ptr(), ptr::null_mut(), flags)
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
	pub fn listen(&mut self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| ErrorKind::AddressInvalid)?;
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_listen(self.inner.handle, addr.as_ptr(), ptr::null_mut(), flags)
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
	/// The default is blocking operations. This setting is _not_ propagated to other handles cloned
	/// from this one.
	pub fn set_nonblocking(&mut self, nonblocking: bool)
	{
		self.nonblocking = nonblocking;
	}

	/// Receives a message from the socket.
	///
	/// The semantics of what receiving a message means vary from protocol to
	/// protocol, so examination of the protocol documentation is encouraged.
	/// For example, with a _req_ socket a message may only be received after a
	/// request has been sent. Furthermore, some protocols may not support
	/// receiving data at all, such as _pub_.
	pub fn recv(&mut self) -> Result<Message>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_recvmsg(self.inner.handle, &mut msgp as _, flags)
		};

		validate_ptr!(rv, msgp);
		Ok(unsafe { Message::from_ptr(msgp) })
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
	pub fn send(&mut self, data: Message) -> SendResult<()>
	{
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		unsafe {
			let msgp = data.into_ptr();
			let rv = nng_sys::nng_sendmsg(self.inner.handle, msgp, flags);

			if rv != 0 {
				Err((Message::from_ptr(msgp), ErrorKind::from_code(rv).into()))
			} else {
				Ok(())
			}
		}
	}

	/// Send a message using the socket asynchronously.
	///
	/// The result of this operation will be available either after calling
	/// `Aio::wait` or inside of the callback function. If the send operation
	/// fails, the message can be retrieved using the `Aio::get_msg` function.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress, this function will return `ErrorKind::TryAgain`
	/// and return the message to the caller.
	pub fn send_async(&mut self, aio: &Aio, msg: Message) -> SendResult<()>
	{
		aio.send_socket(self, msg)
	}

	/// Receive a message using the socket asynchronously.
	///
	/// The result of this operation will be available either after calling
	/// `Aio::wait` or inside of the callback function. If the send operation
	/// fails, the message can be retrieved using the `Aio::get_msg` function.
	///
	/// This function will return immediately. If there is already an I/O
	/// operation in progress that is _not_ a receive operation, this function
	/// will return `ErrorKind::TryAgain`.
	pub fn recv_async(&mut self, aio: &Aio) -> Result<()>
	{
		aio.recv_socket(self)
	}

	/// Close the underlying socket.
	///
	/// Messages that have been submitted for sending may be flushed or delivered depending on the
	/// transport and the linger option. Further attempts to use the socket (via this handle or any
	/// other) after this call returns will result in an error. Threads waiting for operations on
	/// the socket when this call is executed may also return with an error.
	///
	/// Closing the socket while data is in transmission will likely lead to loss of that data.
	/// There is no automatic linger or flush to ensure that the socket send buffers have completely
	/// transmitted. It is recommended to wait a brief period after sending data before calling this
	/// function.
	///
	/// This function will be called automatically when all handles have been dropped.
	pub fn close(self)
	{
		self.inner.close()
	}

	/// Get the positive identifier for the socket.
	pub fn id(&self) -> i32
	{
		let id = unsafe { nng_sys::nng_socket_id(self.inner.handle) };
		assert!(id > 0, "Invalid socket ID returned from valid socket");

		id
	}

	/// Returns the underlying `nng_socket`.
	pub(crate) fn handle(&self) -> nng_sys::nng_socket
	{
		self.inner.handle
	}
}

expose_options!{
	Socket :: inner.handle -> nng_sys::nng_socket;

	GETOPT_BOOL = nng_sys::nng_getopt_bool;
	GETOPT_INT = nng_sys::nng_getopt_int;
	GETOPT_MS = nng_sys::nng_getopt_ms;
	GETOPT_SIZE = nng_sys::nng_getopt_size;
	GETOPT_SOCKADDR = crate::fake_opt;
	GETOPT_STRING = nng_sys::nng_getopt_string;

	SETOPT = nng_sys::nng_setopt;
	SETOPT_BOOL = nng_sys::nng_setopt_bool;
	SETOPT_INT = nng_sys::nng_setopt_int;
	SETOPT_MS = nng_sys::nng_setopt_ms;
	SETOPT_SIZE = nng_sys::nng_setopt_size;
	SETOPT_STRING = nng_sys::nng_setopt_string;

	Gets -> [Raw, MaxTtl, RecvBufferSize,
	         RecvTimeout, SendBufferSize,
	         SendTimeout, SocketName,
	         protocol::reqrep::ResendTime,
	         protocol::survey::SurveyTime];
	Sets -> [ReconnectMinTime, ReconnectMaxTime,
	         RecvBufferSize, RecvMaxSize,
	         RecvTimeout, SendBufferSize,
	         SendTimeout, SocketName, MaxTtl,
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

/// A wrapper type around the underlying `nng_socket`.
///
/// This allows us to have mutliple Rust socket types that won't clone the C
/// socket type before Rust is done with it.
#[derive(Debug)]
struct Inner
{
	/// Handle to the underlying nng socket.
	handle: nng_sys::nng_socket,
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
			rv == 0 || rv == nng_sys::NNG_ECLOSED,
			"Unexpected error code while closing socket ({})", rv
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
