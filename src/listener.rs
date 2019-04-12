//! Nanomsg-next-generation listeners.
//!
//! A listener is the object that is responsible for accepting incoming
//! connections. A given listener can have many connections to multiple clients
//! simultaneously.
//!  Directly creating a listener object is only necessary when one wishes to
//! configure the listener before opening it or if one wants to close the
//! connections without closing the socket. Otherwise, `Socket::listen` can be
//! used.
//!
//! Note that the client/server relationship described by a dialer/listener is
//! completely orthogonal to any similar relationship in the protocols. For
//! example, a _rep_ socket may use a dialer to connect to a listener on a
//! _req_ socket. This orthogonality can lead to innovative solutions to
//! otherwise challenging communications problems.
//!
//! See the [nng documentation][1] for more information.
//!
//! [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_listener.5.html
use std::{cmp, ffi::CString};

use crate::{error::{Error, Result}, socket::Socket};

#[cfg(windows)]
use crate::options::transport::ipc::IpcSecurityDescriptor;

/// A constructed and running listener.
///
/// This listener has already been started on the socket and will continue
/// serving the connection until either it is explicitly close or the owning
/// socket is closed.
#[derive(Clone, Debug)]
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
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let mut handle = nng_sys::nng_listener::NNG_LISTENER_INITIALIZER;
		let flags = if nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_listen(socket.handle(), addr.as_ptr(), &mut handle as *mut _, flags as i32)
		};

		rv2res!(rv, Listener { handle })
	}

	/// Closes the listener.
	///
	/// This also closes any `Pipe` objects that have been created by the
	/// listener. Once this function returns, the listener has been closed and
	/// all of its resources have been deallocated. Therefore, any attempt to
	/// utilize the listener (with this or any other handle) will result in an
	/// error.
	///
	/// Listeners are implicitly closed when the socket they are associated with
	/// is closed. Listeners are _not_ closed when all handles are dropped.
	pub fn close(self)
	{
		// Closing the listener should only ever result in success or ECLOSED
		// and both of those mean that the drop was successful.
		let rv = unsafe { nng_sys::nng_listener_close(self.handle) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED as i32,
			"Unexpected error code while closing listener ({})",
			rv
		);
	}

	/// Returns the positive identifier for the listener.
	pub fn id(&self) -> i32
	{
		let id = unsafe { nng_sys::nng_listener_id(self.handle) };
		assert!(id > 0, "Invalid listener ID returned from valid socket");

		id
	}

	/// Create a new Listener handle from a libnng handle.
	///
	/// This function will panic if the handle is not valid.
	pub(crate) fn from_nng_sys(handle: nng_sys::nng_listener) -> Self
	{
		assert!(
			unsafe { nng_sys::nng_listener_id(handle) > 0},
			"Listener handle is not initialized"
		);
		Listener { handle }
	}
}

impl cmp::PartialEq for Listener
{
	fn eq(&self, other: &Listener) -> bool
	{
		unsafe {
			nng_sys::nng_listener_id(self.handle) == nng_sys::nng_listener_id(other.handle)
		}
	}
}

impl cmp::Eq for Listener {}

#[rustfmt::skip]
expose_options!{
	Listener :: handle -> nng_sys::nng_listener;

	GETOPT_BOOL = nng_sys::nng_listener_getopt_bool;
	GETOPT_INT = nng_sys::nng_listener_getopt_int;
	GETOPT_MS = nng_sys::nng_listener_getopt_ms;
	GETOPT_SIZE = nng_sys::nng_listener_getopt_size;
	GETOPT_SOCKADDR = nng_sys::nng_listener_getopt_sockaddr;
	GETOPT_STRING = nng_sys::nng_listener_getopt_string;

	SETOPT = nng_sys::nng_listener_setopt;
	SETOPT_BOOL = nng_sys::nng_listener_setopt_bool;
	SETOPT_INT = nng_sys::nng_listener_setopt_int;
	SETOPT_MS = nng_sys::nng_listener_setopt_ms;
	SETOPT_PTR = nng_sys::nng_listener_setopt_ptr;
	SETOPT_SIZE = nng_sys::nng_listener_setopt_size;
	SETOPT_STRING = nng_sys::nng_listener_setopt_string;

	Gets -> [LocalAddr, Raw, RecvBufferSize,
	         RecvTimeout, SendBufferSize, Url,
	         SendTimeout, SocketName, MaxTtl,
	         protocol::reqrep::ResendTime,
	         protocol::survey::SurveyTime,
	         transport::tcp::NoDelay,
	         transport::tcp::KeepAlive];
	Sets -> [];
}

/// Configuration utility for nanomsg-next-generation listeners.
///
/// This object allows for the configuration of listeners before they are
/// started. If it is not necessary to change listener settings or to close the
/// listener without closing the socket, then `Socket::listen` provides a
/// simpler interface and does not require tracking an object.
#[derive(Debug)]
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
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let mut handle = nng_sys::nng_listener::NNG_LISTENER_INITIALIZER;
		let rv = unsafe {
			nng_sys::nng_listener_create(&mut handle as *mut _, socket.handle(), addr.as_ptr())
		};

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
	pub fn start(self, nonblocking: bool) -> std::result::Result<Listener, (Self, Error)>
	{
		let flags = if nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		// If there is an error starting the listener, we don't want to consume
		// it. Instead, we'll return it to the user and they can decide what to
		// do.
		let rv = unsafe { nng_sys::nng_listener_start(self.handle, flags as i32) };

		match rv {
			0 => {
				let handle = Listener { handle: self.handle };
				std::mem::forget(self);
				Ok(handle)
			},
			e => Err((self, Error::from_code(e as u32))),
		}
	}
}

#[rustfmt::skip]
expose_options!{
	ListenerOptions :: handle -> nng_sys::nng_listener;

	GETOPT_BOOL = nng_sys::nng_listener_getopt_bool;
	GETOPT_INT = nng_sys::nng_listener_getopt_int;
	GETOPT_MS = nng_sys::nng_listener_getopt_ms;
	GETOPT_SIZE = nng_sys::nng_listener_getopt_size;
	GETOPT_SOCKADDR = nng_sys::nng_listener_getopt_sockaddr;
	GETOPT_STRING = nng_sys::nng_listener_getopt_string;

	SETOPT = nng_sys::nng_listener_setopt;
	SETOPT_BOOL = nng_sys::nng_listener_setopt_bool;
	SETOPT_INT = nng_sys::nng_listener_setopt_int;
	SETOPT_MS = nng_sys::nng_listener_setopt_ms;
	SETOPT_PTR = nng_sys::nng_listener_setopt_ptr;
	SETOPT_SIZE = nng_sys::nng_listener_setopt_size;
	SETOPT_STRING = nng_sys::nng_listener_setopt_string;

	Gets -> [LocalAddr, Raw, RecvBufferSize,
	         RecvTimeout, SendBufferSize, Url,
	         SendTimeout, SocketName, MaxTtl,
	         protocol::reqrep::ResendTime,
	         protocol::survey::SurveyTime,
	         transport::tcp::NoDelay,
	         transport::tcp::KeepAlive];
	Sets -> [RecvMaxSize, transport::tcp::NoDelay,
	         transport::tcp::KeepAlive,
	         transport::tls::CaFile,
	         transport::tls::CertKeyFile,
	         transport::websocket::ResponseHeaders];
}

#[cfg(windows)]
impl crate::options::UnsafeSetOpt<IpcSecurityDescriptor> for ListenerOptions {}

impl Drop for ListenerOptions
{
	fn drop(&mut self)
	{
		// Closing the listener should only ever result in success or ECLOSED
		// and both of those mean that the drop was successful.
		let rv = unsafe { nng_sys::nng_listener_close(self.handle) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED as i32,
			"Unexpected error code while closing listener ({})",
			rv
		);
	}
}
