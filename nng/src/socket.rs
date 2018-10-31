use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;
use nng_sys::protocol::*;
use crate::error::{ErrorKind, Result, SendResult};
use crate::message::Message;

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
/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_socket.5.html
#[derive(Debug)]
pub struct Socket
{
	/// Handle to the underlying nng socket.
	handle: nng_sys::nng_socket,

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
				Protocol::Push0 => pipeline0::nng_pull0_open(&mut socket as *mut _),
				Protocol::Rep0 => reqrep0::nng_rep0_open(&mut socket as *mut _),
				Protocol::Req0 => reqrep0::nng_req0_open(&mut socket as *mut _),
				Protocol::Respondent0 => survey0::nng_respondent0_open(&mut socket as *mut _),
				Protocol::Sub0 => pubsub0::nng_sub0_open(&mut socket as *mut _),
				Protocol::Surveyor0 => survey0::nng_surveyor0_open(&mut socket as *mut _),
			}
		};

		rv2res!(rv, Socket { handle: socket, nonblocking: false })
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
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_dial.3.html
	pub fn dial(&mut self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| ErrorKind::AddressInvalid)?;
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_dial(self.handle, addr.as_ptr(), ptr::null_mut(), flags)
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
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_listen.3.html
	pub fn listen(&mut self, url: &str) -> Result<()>
	{
		let addr = CString::new(url).map_err(|_| ErrorKind::AddressInvalid)?;
		let flags = if self.nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_listen(self.handle, addr.as_ptr(), ptr::null_mut(), flags)
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
	/// The default is blocking operations.
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
			nng_sys::nng_recvmsg(self.handle, &mut msgp as _, flags)
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

		let rv = unsafe {
			nng_sys::nng_sendmsg(self.handle, data.msgp(), flags)
		};

		if rv != 0 {
			Err((data, ErrorKind::from_code(rv).into()))
		} else {
			// If the message was sent, we no longer have ownership over the
			// memory. Forget that it was a thing and move on.
			std::mem::forget(data);
			Ok(())
		}
	}

	/// Get the positive identifier for the socket.
	pub fn id(&self) -> i32
	{
		let id = unsafe { nng_sys::nng_socket_id(self.handle) };
		assert!(id > 0, "Invalid socket ID returned from valid socket");

		id
	}

	/// Returns the underlying `nng_socket`.
	pub(crate) fn handle(&self) -> nng_sys::nng_socket
	{
		self.handle
	}
}

expose_options!{
	Socket :: handle -> nng_sys::nng_socket;

	GETOPT_BOOL = nng_sys::nng_getopt_bool;
	GETOPT_INT = nng_sys::nng_getopt_int;
	GETOPT_MS = nng_sys::nng_getopt_ms;
	GETOPT_SIZE = nng_sys::nng_getopt_size;
	GETOPT_SOCKADDR = fake_getopt_sockaddr;
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

/// Fake function for getting the address of a Socket.
///
/// Nng does not support any options that get or set a SockAddr. So we must
/// fake one in order to be able to implement the options trait. Since we're in
/// full control and this should never be called, we panic.
extern "C" fn fake_getopt_sockaddr(_: nng_sys::nng_socket, _: *const c_char, _: *mut nng_sys::nng_sockaddr) -> c_int
{
	panic!("Socket has no `sockaddr` options");
}

impl Drop for Socket
{
	fn drop(&mut self)
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

/// Protocols available for use by sockets.
#[derive(Debug)]
pub enum Protocol
{
	/// Version 0 of the bus protocol.
	///
	/// The _bus_ protocol provides for building mesh networks where every peer
	/// is connected to every other peer. In this protocol, each message sent
	/// by a node is sent to every one of its directly connected peers. See
	/// the [bus documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_bus.7.html
	Bus0,

	/// Version 0 of the pair protocol.
	///
	/// The _pair_ protocol implements a peer-to-peer pattern, where
	/// relationships between peers are one-to-one. See the
	/// [pair documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_pair.7.html
	Pair0,

	/// Version 1 of the pair protocol.
	///
	/// The _pair_ protocol implements a peer-to-peer pattern, where
	/// relationships between peers are one-to-one. Version 1 of this protocol
	/// supports and optional _polyamorous_ mode. See the [pair documentation][1]
	/// for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_pair.7.html
	Pair1,

	/// Version 0 of the publisher protocol.
	///
	/// The _pub_ protocol is one half of a publisher/subscriber pattern. In
	/// this pattern, a publisher sends data, which is broadcast to all
	/// subscribers. The subscribing applications only see the data to which
	/// they have subscribed. See the [publisher/subscriber documentation][1]
	/// for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_pub.7.html
	Pub0,

	/// Version 0 of the pull protocol.
	///
	/// The _pull_ protocol is one half of a pipeline pattern. The other half
	/// is the _push_ protocol. In the pipeline pattern, pushers distribute
	/// messages to pullers. Each message sent by a pusher will be sent to one
	/// of its peer pullers, chosen in a round-robin fashion from the set of
	/// connected peers available for receiving. This property makes this
	/// pattern useful in load-balancing scenarios.
	///
	/// See the [pipeline documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_pull.7.html
	Pull0,

	/// Version 0 of the push protocol.
	///
	/// The _push_ protocol is one half of a pipeline pattern. The other side
	/// is the _pull_ protocol. In the pipeline pattern, pushers distribute
	/// messages to pullers. Each message sent by a pusher will be sent to one
	/// of its peer pullers, chosen in a round-robin fashion from the set of
	/// connected peers available for receiving. This property makes this
	/// pattern useful in load-balancing scenarios.
	///
	/// See the [pipeline documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_push.7.html
	Push0,

	/// Version 0 of the reply protocol.
	///
	/// The _rep_ protocol is one half of a request/reply pattern. In this
	/// pattern, a requester sends a message to one replier, who is expected to
	/// reply. The request is resent if no reply arrives, until a reply is
	/// received or the request times out.
	///
	/// See the [request/reply documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_rep.7.html
	Rep0,

	/// Version 0 of the request protocol.
	///
	/// The _req_ protocol is one half of a request/reply pattern. In this
	/// pattern, a requester sends a message to one replier, who is expected to
	/// reply. The request is resent if no reply arrives, until a reply is
	/// received or the request times out.
	///
	/// See the [request/reply documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_req.7.html
	Req0,

	/// Version 0 of the respondent protocol.
	///
	/// The _respondent_ protocol is one half of a survey pattern. In this
	/// pattern, a surveyor sends a survey, which is broadcast to all peer
	/// respondents. The respondents then have a chance to reply (but are not
	/// obliged to reply). The survey itself is a timed event, so that
	/// responses received after the survey has finished are discarded.
	///
	/// See the [survery documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_respondent.7.html
	Respondent0,

	/// Version 0 of the subscriber protocol.
	///
	/// The _sub_ protocol is one half of a publisher/subscriber pattern. In
	/// this pattern, a publisher sends data, which is broadcast to all
	/// subscribers. The subscribing applications only see the data to which
	/// they have subscribed.
	///
	/// See the [publisher/subscriber documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_sub.7.html
	Sub0,

	/// Version 0 of the surveyor protocol.
	///
	/// The _surveyor_ protocol is one half of a survey pattern. In this
	/// pattern, a surveyor sends a survey, which is broadcast to all peer
	/// respondents. The respondents then have a chance to reply (but are not
	/// obliged to reply). The survey itself is a timed event, so that
	/// responses received after the survey has finished are discarded.
	///
	/// See the [survey documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_surveyor.7.html
	Surveyor0,
}
