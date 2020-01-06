use std::{
	cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
	ffi::CString,
	hash::{Hash, Hasher},
	num::NonZeroU32,
};

use crate::{
	error::{Error, Result},
	socket::Socket,
};

/// Active listener for incoming connections.
///
/// A `Listener` is the object that is responsible for accepting incoming
/// connections. A given `Listener` can have many connections to multiple clients
/// simultaneously. Directly creating a listener object is only necessary when one wishes to
/// configure the listener before opening it or if one wants to close the
/// connections without closing the socket. Otherwise, [`Socket::listen`] can be
/// used.
///
/// Note that the client/server relationship described by a dialer/listener is
/// completely orthogonal to any similar relationship in the protocols. For
/// example, a _rep_ socket may use a dialer to connect to a listener on a
/// _req_ socket. This orthogonality can lead to innovative solutions to
/// otherwise challenging communications problems.
///
/// See the [NNG documentation][1] for more information.
///
///
/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_listener.5.html
/// [`Socket::listen`]: struct.Socket.html#method.listen
#[derive(Clone, Copy, Debug)]
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
	/// will be possible. Use [`ListenerOptions`] to change the listener options
	/// before starting it.
	///
	/// # Errors
	///
	/// * [`AddressInUse`]: The address specified by _url_ is already in use.
	/// * [`Addressinvalid`]: An invalid _url_ was specified.
	/// * [`Closed`]: The socket is not open.
	/// * [`OutOfMemory`]: Insufficient memory is available.
	///
	///
	/// [`AddressInUse`]: enum.Error.html#variant.AddressInUse
	/// [`Addressinvalid`]: enum.Error.html#variant.Addressinvalid
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`ListenerOptions`]: struct.ListenerOptions.html
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	pub fn new(socket: &Socket, url: &str) -> Result<Self>
	{
		// We take a Rust string instead of a c-string because the cost of
		// creating the listener will far outweigh the cost of allocating a
		// single string. Having a full Rust interface will make it easier to
		// work with.
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let mut handle = nng_sys::nng_listener::NNG_LISTENER_INITIALIZER;

		let rv = unsafe {
			nng_sys::nng_listen(socket.handle(), addr.as_ptr(), &mut handle as *mut _, 0)
		};

		rv2res!(rv, Listener { handle })
	}

	/// Closes the listener.
	///
	/// This also closes any [`Pipe`] objects that have been created by the
	/// listener. Once this function returns, the listener has been closed and
	/// all of its resources have been deallocated. Therefore, any attempt to
	/// utilize the listener (with this or any other handle) will result in an
	/// error.
	///
	/// Listeners are implicitly closed when the socket they are associated with
	/// is closed. Listeners are _not_ closed when all handles are dropped.
	///
	///
	/// [`Pipe`]: struct.Pipe.html
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

	/// Create a new `Listener` handle from a NNG handle.
	///
	/// This function will panic if the handle is not valid.
	pub(crate) fn from_nng_sys(handle: nng_sys::nng_listener) -> Self
	{
		assert!(
			unsafe { nng_sys::nng_listener_id(handle) > 0 },
			"Listener handle is not initialized"
		);
		Listener { handle }
	}
}

#[cfg(feature = "ffi-module")]
impl Listener
{
	/// Returns the underlying `nng_listener` object.
	pub fn nng_listener(self) -> nng_sys::nng_listener { self.handle }
}

impl PartialEq for Listener
{
	fn eq(&self, other: &Listener) -> bool
	{
		unsafe { nng_sys::nng_listener_id(self.handle) == nng_sys::nng_listener_id(other.handle) }
	}
}

impl Eq for Listener {}

impl PartialOrd for Listener
{
	fn partial_cmp(&self, other: &Listener) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Listener
{
	fn cmp(&self, other: &Listener) -> Ordering
	{
		unsafe {
			let us = nng_sys::nng_listener_id(self.handle);
			let them = nng_sys::nng_listener_id(other.handle);
			us.cmp(&them)
		}
	}
}

impl Hash for Listener
{
	fn hash<H: Hasher>(&self, state: &mut H)
	{
		let id = unsafe { nng_sys::nng_listener_id(self.handle) };
		id.hash(state)
	}
}

#[rustfmt::skip]
expose_options!{
	Listener :: handle -> nng_sys::nng_listener;

	GETOPT_BOOL = nng_sys::nng_listener_get_bool;
	GETOPT_INT = nng_sys::nng_listener_get_int;
	GETOPT_MS = nng_sys::nng_listener_get_ms;
	GETOPT_SIZE = nng_sys::nng_listener_get_size;
	GETOPT_SOCKADDR = nng_sys::nng_listener_get_addr;
	GETOPT_STRING = nng_sys::nng_listener_get_string;
	GETOPT_UINT64 = nng_sys::nng_listener_get_uint64;

	SETOPT = nng_sys::nng_listener_set;
	SETOPT_BOOL = nng_sys::nng_listener_set_bool;
	SETOPT_INT = nng_sys::nng_listener_set_int;
	SETOPT_MS = nng_sys::nng_listener_set_ms;
	SETOPT_PTR = nng_sys::nng_listener_set_ptr;
	SETOPT_SIZE = nng_sys::nng_listener_set_size;
	SETOPT_STRING = nng_sys::nng_listener_set_string;

	Gets -> [LocalAddr, Raw, RecvBufferSize,
	         RecvTimeout, SendBufferSize, Url,
	         SendTimeout, SocketName, MaxTtl,
	         protocol::reqrep::ResendTime,
	         protocol::survey::SurveyTime,
	         transport::tcp::NoDelay,
	         transport::tcp::KeepAlive,
	         transport::tcp::BoundPort,
	         transport::websocket::Protocol];
	Sets -> [];
}

/// Configuration utility for nanomsg-next-generation listeners.
///
/// This object allows for the configuration of listeners before they are
/// started. If it is not necessary to change listener settings or to close the
/// listener without closing the socket, then [`Socket::listen`] provides a
/// simpler interface.
///
///
/// [`Socket::listen`]: struct.Socket.html#method.listen
#[derive(Debug)]
pub struct ListenerOptions
{
	/// The underlying listener object that we are configuring
	handle: nng_sys::nng_listener,
}
impl ListenerOptions
{
	/// Creates a new [`Listener`] object associated with the given socket.
	///
	/// Note that this does not start the [`Listener`] In order to start the
	/// listener, this object must be consumed by [`ListenerOptions::start`].
	///
	/// # Errors
	///
	/// * [`AddressInvalid`]: An invalid _url_ was specified.
	/// * [`Closed`]: The socket is not open.
	/// * [`OutOfMemory`]: Insufficient memory.
	///
	///
	/// [`AddressInvalid`]: enum.Error.html#variant.AddressInvalid
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`Listener`]: struct.Listener.html
	/// [`ListenerOptions::start`]: struct.ListenerOptions.html#method.start
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
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

	/// Cause the [`Listener`] to start listening on the address with which it was
	/// created.
	///
	/// The returned handle controls the life of the [`Listener`]. If it is
	/// dropped, the [`Listener`] is shut down and no more messages will be
	/// received on it.
	///
	/// # Errors
	///
	/// * [`Closed`]: The socket is not open.
	///
	///
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`Listener`]: struct.Listener.html
	pub fn start(self) -> std::result::Result<Listener, (Self, Error)>
	{
		// If there is an error starting the listener, we don't want to consume
		// it. Instead, we'll return it to the user and they can decide what to
		// do.
		let rv = unsafe { nng_sys::nng_listener_start(self.handle, 0) };

		if let Some(e) = NonZeroU32::new(rv as u32) {
			Err((self, Error::from(e)))
		}
		else {
			let handle = Listener { handle: self.handle };
			std::mem::forget(self);
			Ok(handle)
		}
	}
}

#[cfg(feature = "ffi-module")]
impl ListenerOptions
{
	/// Returns the underlying `nng_listener` object.
	pub fn nng_listener(&self) -> nng_sys::nng_listener { self.handle }
}

#[rustfmt::skip]
expose_options!{
	ListenerOptions :: handle -> nng_sys::nng_listener;

	GETOPT_BOOL = nng_sys::nng_listener_get_bool;
	GETOPT_INT = nng_sys::nng_listener_get_int;
	GETOPT_MS = nng_sys::nng_listener_get_ms;
	GETOPT_SIZE = nng_sys::nng_listener_get_size;
	GETOPT_SOCKADDR = nng_sys::nng_listener_get_addr;
	GETOPT_STRING = nng_sys::nng_listener_get_string;
	GETOPT_UINT64 = nng_sys::nng_listener_get_uint64;

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
	         transport::tcp::KeepAlive,
	         transport::websocket::Protocol];
	Sets -> [RecvMaxSize, transport::tcp::NoDelay,
	         transport::tcp::KeepAlive,
	         transport::tls::CaFile,
	         transport::tls::CertKeyFile,
	         transport::websocket::ResponseHeaders,
	         transport::websocket::Protocol];
}

#[cfg(unix)]
mod unix_impls
{
	use super::*;
	use crate::options::transport::ipc;

	impl crate::options::SetOpt<ipc::Permissions> for ListenerOptions {}
}

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
