use std::{mem, ptr};
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use nng_sys;

use error::{Error, ErrorKind, Result};
use socket::Socket;
use addr::SocketAddr;

/// A nanomsg-next-generation listener.
///
/// A listener is the object that is responsible for accepting incoming
/// connections. A given listener can have many connections to multiple clients
/// simultaneously.
///
/// Directly creating a listener object is only necessary when one wishes to
/// configure the listener before opening it or if one wants to close the
/// connections without closing the socket. Otherwise, `Socket::listen` can be
/// used.
///
/// Note that the client/server relationship described by a dialer/listener is
/// completely orthogonal to any similar relationship in the protocols. For
/// example, a _rep_ socket may use a dialer to connect to a listener on a
/// _req_ socket. This orthogonality can lead to innovative solutions to
/// otherwise challenging communications problems.
///
/// See the [nng documentation][1] for more information.
///
/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_listener.5.html
pub struct Listener
{
	/// The handle to the underlying
	handle: nng_sys::nng_listener,
}
impl Listener
{
	/// Creates a new listener object associated with the given socket.
	///
	/// Note that this will immediately start the listener so no configuration
	/// will be possible. Use `ListenerOptions` to change the listener options
	/// before starting it.
	pub fn new(socket: &Socket, url: &str, nonblocking: bool) -> Result<Self>
	{
		// We take a Rust string instead of a c-string because the cost of
		// creating the listener will far outweigh the cost of allocating a
		// single string. Having a full Rust interface will make it easier to
		// work with.
		let addr = CString::new(url).map_err(|_| ErrorKind::AddressInvalid)?;
		let mut handle = nng_sys::NNG_LISTENER_INITIALIZER;
		let flags = if nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_listen(socket.handle, addr.as_ptr(), &mut handle as *mut _, flags)
		};

		rv2res!(rv, Listener { handle })
	}

	/// Returns the positive identifier for the listener.
	pub fn id(&self) -> i32
	{
		let id = unsafe { nng_sys::nng_listener_id(self.handle) };
		assert!(id > 0, "Invalid listener ID returned from valid socket");

		id
	}
}

impl Listener
{
	/// The local address used for communication.
	///
	/// Not all transports support this option and some transports support it
	/// for listeners but not dialers.
	pub fn local_addr(&self) -> Result<SocketAddr>
	{
		unsafe {
			let mut addr: nng_sys::nng_sockaddr = mem::uninitialized();
			let rv = nng_sys::nng_listener_getopt_sockaddr(self.handle, nng_sys::NNG_OPT_LOCADDR, &mut addr as _);

			rv2res!(rv, addr.into())
		}
	}

	/// The maximum message size that will be accepted from a remote peer.
	///
	/// If a peer attempts to send a message larger than this, then the message
	/// will be discarded. If the value of this is zero, then no limit on
	/// message sizes is enforced.
	pub fn recv_max_size(&self) -> Result<usize>
	{
		let mut sz: usize = 0;
		let rv = unsafe {
			nng_sys::nng_listener_getopt_size(self.handle, nng_sys::NNG_OPT_RECVMAXSZ, &mut sz as _)
		};

		rv2res!(rv, sz)
	}

	/// The URL with which the listener was configured.
	///
	/// Some transports will canonify URLs before returning them to the
	/// application.
	pub fn url(&self) -> Result<String>
	{
		unsafe {
			let mut ptr: *mut c_char = ptr::null_mut();
			let rv = nng_sys::nng_listener_getopt_string(self.handle, nng_sys::NNG_OPT_URL, &mut ptr as *mut _);

			if rv != 0 {
				return Err(ErrorKind::from_code(rv).into());
			}

			assert!(ptr != ptr::null_mut(), "Nng returned a null pointer from a successful function");
			let url = CStr::from_ptr(ptr).to_string_lossy().into_owned();
			nng_sys::nng_strfree(ptr);

			Ok(url)
		}
	}

	/// Whether or not Nagle's algorithm is enabled for TCP connections.
	///
	/// When `true` (the default), messages are sent immediately by the
	/// underlying TCP stream without waiting to gather more data. When
	/// `false`, Nagle's algorithm is enabled and the TCP stream may wait
	/// briefly in an attempt to coalesce messages. Nagle's algorithm is useful
	/// on low-bandwidth connections to reduce overhead but it comes at a cost
	/// to latency.
	pub fn tcp_nodelay(&self) -> Result<bool>
	{
		let mut enabled = true;
		let rv = unsafe {
			nng_sys::nng_listener_getopt_bool(self.handle, nng_sys::NNG_OPT_TCP_NODELAY, &mut enabled as _)
		};

		rv2res!(rv, enabled)
	}

	/// Whether or not keep-alive messages are enabled on the underlying TCP stream.
	///
	/// This option is `false` by default. When enabled, if no messages are
	/// seen for a period of time, then a zero length TCP message is sent with
	/// the ACK flag set in an attempt to tickle some traffic from the peer. If
	/// none is still seen (after some platform-specific number of retries and
	/// timeouts), then the remote peer is presumed dead and the connection is
	/// closed.
	pub fn tcp_keepalive(&self) -> Result<bool>
	{
		let mut enabled = false;
		let rv = unsafe {
			nng_sys::nng_listener_getopt_bool(self.handle, nng_sys::NNG_OPT_TCP_KEEPALIVE, &mut enabled as _)
		};

		rv2res!(rv, enabled)
	}
}

impl Drop for Listener
{
	fn drop(&mut self)
	{
		// Closing the listener should only ever result in success or ECLOSED
		// and both of those mean that the drop was successful.
		let rv = unsafe { nng_sys::nng_listener_close(self.handle) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED,
			"Unexpected error code while closing listener ({})", rv
		);
	}
}

/// Configuration utility for nanomsg-next-generation listeners.
///
/// This object allows for the configuration of listeners before they are
/// started. If it is not necessary to change listener settings or to close the
/// listener without closing the socket, then `Socket::listen` provides a simpler
/// interface and does not require tracking an object.
pub struct ListenerOptions
{
	/// The underlying listener object that we are configuring
	handle: nng_sys::nng_listener,
}
impl ListenerOptions
{
	/// Creates a new listener object associated with the given socket.
	///
	/// Note that this does not start the listener. In order to start the
	/// listener, this object must be consumed by `ListenerOptions::start`.
	pub fn new(socket: &Socket, url: &str) -> Result<Self>
	{
		// We take a Rust string instead of a c-string because the cost of
		// creating the listener will far outweigh the cost of allocating a
		// single string. Having a full Rust interface will make it easier to
		// work with.
		let addr = CString::new(url).map_err(|_| ErrorKind::AddressInvalid)?;
		let mut handle = nng_sys::NNG_LISTENER_INITIALIZER;
		let rv = unsafe { nng_sys::nng_listener_create(&mut handle as *mut _, socket.handle, addr.as_ptr()) };

		rv2res!(rv, ListenerOptions { handle })
	}

	/// Cause the listener to start listening on the address with which it was
	/// created.
	///
	/// Normally, the act of "binding" to the address indicated by _url_ is
	/// done synchronously, including any necessary name resolution. As a
	/// result, a failure, such as if the address is already in use, will be
	/// returned immediately. However, if `nonblocking` is specified then this
	/// is done asynchronously; furthermore any failure to bind will be
	/// periodically reattempted in the background.
	///
	/// The returned handle controls the life of the listener. If it is
	/// dropped, the listener is shut down and no more messages will be
	/// received on it.
	pub fn start(self, nonblocking: bool) -> ::std::result::Result<Listener, (Self, Error)>
	{
		let flags = if nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		// If there is an error starting the listener, we don't want to consume
		// it. Instead, we'll return it to the user and they can decide what to
		// do.
		let rv = unsafe {
			nng_sys::nng_listener_start(self.handle, flags)
		};

		match rv {
			0 => {
				let handle = Listener { handle: self.handle };
				mem::forget(self);
				Ok(handle)
			},
			e => Err((self, ErrorKind::from_code(e).into())),
		}
	}
}

impl ListenerOptions
{
	/// Set the maximum message size that will be accepted from a remote peer.
	///
	/// If a peer attempts to send a message larger than this, then the message
	/// will be discarded. If the value of this is zero, then no limit on
	/// message sizes is enforced.
	pub fn recv_max_size(&mut self, sz: usize) -> Result<&mut Self>
	{
		let rv = unsafe {
			nng_sys::nng_listener_setopt_size(self.handle, nng_sys::NNG_OPT_RECVMAXSZ, sz)
		};

		rv2res!(rv, self)
	}

	/// Enable or disable the use of Nagle's algorithm for TCP connections.
	///
	/// When `true` (the default), messages are sent immediately by the
	/// underlying TCP stream without waiting to gather more data. When
	/// `false`, Nagle's algoirthm is enabled and the TCP stream may wait
	/// briefly in an attempt to coalesce messages. Nagle's algorithm is useful
	/// on low-bandwidth connections to reduce overhead but it comes at a cost
	/// to latency.
	pub fn tcp_nodelay(&mut self, enable: bool) -> Result<&mut Self>
	{
		let rv = unsafe {
			nng_sys::nng_listener_setopt_bool(self.handle, nng_sys::NNG_OPT_TCP_NODELAY, enable)
		};

		rv2res!(rv, self)
	}

	/// Enable the sending of keep-alive messages on the underlying TCP stream.
	///
	/// This option is `false` by default. When enabled, if no messages are
	/// seen for a period of time, then a zero length TCP message is sent with
	/// the ACK flag set in an attempt to tickle some traffic from the peer. If
	/// none is still seen (after some platform-specific number of retries and
	/// timeouts), then the remote peer is presumed dead and the connection is
	/// closed.
	pub fn tcp_keepalive(&mut self, enable: bool) -> Result<&mut Self>
	{
		let rv = unsafe {
			nng_sys::nng_listener_setopt_bool(self.handle, nng_sys::NNG_OPT_TCP_KEEPALIVE, enable)
		};

		rv2res!(rv, self)
	}
}

impl Drop for ListenerOptions
{
	fn drop(&mut self)
	{
		// Closing the listener should only ever result in success or ECLOSED
		// and both of those mean that the drop was successful.
		let rv = unsafe { nng_sys::nng_listener_close(self.handle) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED,
			"Unexpected error code while closing listener ({})", rv
		);
	}
}
