//! Nanomsg-next-generation dialers.
//!
//! A dialer is responsible for establishing and maintaining outgoing
//! connections. If a connection is ever broken, or fails, the dialer object
//! automatically attempts to reconnect.
//!
//! Directly creating a dialer object is only necessary when one wishes to
//! configure the connection before opening it or if one wants to close the
//! outgoing connection without closing the socket. Otherwise, `Socket::dial`
//! can be used.
//!
//! Note that the client/server relationship described by a dialer/listener is
//! completely orthogonal to any similar relationship in the protocols. For
//! example, a _rep_ socket may use a dialer to connect to a listener on a
//! _req_ socket. This orthogonality can lead to innovative solutions to
//! otherwise challenging communications problems.
//!
//! See the [nng documentation][1] for more information.
//!
//! [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_dialer.5.html
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

/// A constructed and running dialer.
///
/// This dialer has already been started on the socket and will continue
/// serving the connection until either it is explicitly closed or the owning
/// socket is closed.
#[derive(Clone, Copy, Debug)]
pub struct Dialer
{
	/// The handle to the underlying
	handle: nng_sys::nng_dialer,
}
impl Dialer
{
	/// Creates a new dialer object associated with the given socket.
	///
	/// Note that this will immediately start the dialer so no configuration
	/// will be possible. Use `DialerOptions` to change the dialer options
	/// before starting it.
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
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_dial.3.html
	/// [`AddressInvalid`]: enum.Error.html#variant.AddressInvalid
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`ConnectionRefused`]: enum.Error.html#variant.ConnectionRefused
	/// [`ConnectionReset`]: enum.Error.html#variant.ConnectionReset
	/// [`DestUnreachable`]: enum.Error.html#variant.DestUnreachable
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	/// [`PeerAuth`]: enum.Error.html#variant.PeerAuth
	/// [`Protocol`]: enum.Error.html#variant.Protocol
	pub fn new(socket: &Socket, url: &str, nonblocking: bool) -> Result<Self>
	{
		// We take a Rust string instead of a c-string because the cost of
		// creating the dialer will far outweigh the cost of allocating a
		// single string. Having a full Rust interface will make it easier to
		// work with.
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let mut handle = nng_sys::nng_dialer::NNG_DIALER_INITIALIZER;
		let flags = if nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		let rv = unsafe {
			nng_sys::nng_dial(socket.handle(), addr.as_ptr(), &mut handle as *mut _, flags as i32)
		};

		rv2res!(rv, Dialer { handle })
	}

	/// Closes the dialer.
	///
	/// This also closes any `Pipe` objects that have been created by the
	/// dialer. Once this function returns, the dialer has been closed and all
	/// of its resources have been deallocated. Therefore, any attempt to
	/// utilize the dialer (with this or any other handle) will result in
	/// an error.
	///
	/// Dialers are implicitly closed when the socket they are associated with
	/// is closed. Dialers are _not_ closed when all handles are dropped.
	pub fn close(self)
	{
		// Closing the dialer should only ever result in success or ECLOSED and
		// both of those mean that the drop was successful.
		let rv = unsafe { nng_sys::nng_dialer_close(self.handle) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED as i32,
			"Unexpected error code while closing dialer ({})",
			rv
		);
	}

	/// Create a new Dialer handle from a libnng handle.
	///
	/// This function will panic if the handle is not valid.
	pub(crate) fn from_nng_sys(handle: nng_sys::nng_dialer) -> Self
	{
		assert!(unsafe { nng_sys::nng_dialer_id(handle) > 0 }, "Dialer handle is not initialized");
		Dialer { handle }
	}
}

#[cfg(feature = "ffi-module")]
impl Dialer
{
	/// Returns the underlying `nng_dialer` object.
	pub fn nng_dialer(self) -> nng_sys::nng_dialer { self.handle }
}

impl PartialEq for Dialer
{
	fn eq(&self, other: &Dialer) -> bool
	{
		unsafe { nng_sys::nng_dialer_id(self.handle) == nng_sys::nng_dialer_id(other.handle) }
	}
}

impl Eq for Dialer {}

impl PartialOrd for Dialer
{
	fn partial_cmp(&self, other: &Dialer) -> Option<Ordering> { Some(self.cmp(other)) }
}

impl Ord for Dialer
{
	fn cmp(&self, other: &Dialer) -> Ordering
	{
		unsafe {
			let us = nng_sys::nng_dialer_id(self.handle);
			let them = nng_sys::nng_dialer_id(other.handle);
			us.cmp(&them)
		}
	}
}

impl Hash for Dialer
{
	fn hash<H: Hasher>(&self, state: &mut H)
	{
		let id = unsafe { nng_sys::nng_dialer_id(self.handle) };
		id.hash(state)
	}
}

#[rustfmt::skip]
expose_options!{
	Dialer :: handle -> nng_sys::nng_dialer;

	GETOPT_BOOL = nng_sys::nng_dialer_get_bool;
	GETOPT_INT = nng_sys::nng_dialer_get_int;
	GETOPT_MS = nng_sys::nng_dialer_get_ms;
	GETOPT_SIZE = nng_sys::nng_dialer_get_size;
	GETOPT_SOCKADDR = nng_sys::nng_dialer_get_addr;
	GETOPT_STRING = nng_sys::nng_dialer_get_string;
	GETOPT_UINT64 = nng_sys::nng_dialer_get_uint64;

	SETOPT = nng_sys::nng_dialer_set;
	SETOPT_BOOL = nng_sys::nng_dialer_set_bool;
	SETOPT_INT = nng_sys::nng_dialer_set_int;
	SETOPT_MS = nng_sys::nng_dialer_set_ms;
	SETOPT_PTR = nng_sys::nng_dialer_set_ptr;
	SETOPT_SIZE = nng_sys::nng_dialer_set_size;
	SETOPT_STRING = nng_sys::nng_dialer_set_string;

	Gets -> [LocalAddr, Raw, ReconnectMinTime,
	         ReconnectMaxTime, RecvBufferSize,
	         RecvMaxSize, RecvTimeout,
	         SendBufferSize, SendTimeout,
	         SocketName, MaxTtl, Url,
	         protocol::reqrep::ResendTime,
	         protocol::survey::SurveyTime,
	         transport::tcp::NoDelay,
	         transport::tcp::KeepAlive,
	         transport::websocket::Protocol];
	Sets -> [];
}

/// Configuration utility for nanomsg-next-generation dialers.
///
/// This object allows for the configuration of dialers before they are
/// started. If it is not necessary to change dialer settings or to close the
/// dialer without closing the socket, then `Socket::dial` provides a simpler
/// interface and does not require tracking an object.
#[derive(Debug)]
pub struct DialerOptions
{
	/// The underlying dialer object that we are configuring
	handle: nng_sys::nng_dialer,
}
impl DialerOptions
{
	/// Creates a new dialer object associated with the given socket.
	///
	/// Note that this does not start the dialer. In order to start the dialer,
	/// this object must be consumed by `DialerOptions::start`.
	///
	/// # Errors
	///
	/// * [`AddressInvalid`]: An invalid _url_ was specified.
	/// * [`Closed`]: The socket is not open.
	/// * [`OutOfMemory`]: Insufficient memory available.
	///
	///
	/// [`AddressInvalid`]: enum.Error.html#variant.AddressInvalid
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	pub fn new(socket: &Socket, url: &str) -> Result<Self>
	{
		// We take a Rust string instead of a c-string because the cost of
		// creating the dialer will far outweigh the cost of allocating a
		// single string. Having a full Rust interface will make it easier to
		// work with.
		let addr = CString::new(url).map_err(|_| Error::AddressInvalid)?;
		let mut handle = nng_sys::nng_dialer::NNG_DIALER_INITIALIZER;
		let rv = unsafe {
			nng_sys::nng_dialer_create(&mut handle as *mut _, socket.handle(), addr.as_ptr())
		};

		rv2res!(rv, DialerOptions { handle })
	}

	/// Cause the dialer to start connecting to the address with which it was
	/// created.
	///
	/// Normally, the first attempt to connect to the dialer's address is done
	/// synchronously, including any necessary name resolution. As a result, a
	/// failure, such as if the connection is refused, will be returned
	/// immediately, and no further action will be taken.
	///
	/// However, if `nonblocking` is specified, then the connection attempt is
	/// made asynchronously.
	///
	/// Furthermore, if the connection was closed for a synchronously dialed
	/// connection, the dialer will still attempt to redial asynchronously.
	///
	/// The returned handle controls the life of the dialer. If it is dropped,
	/// the dialer is shut down and no more messages will be received on it.
	///
	/// # Errors
	///
	/// * [`Closed`]: The socket is not open.
	/// * [`ConnectionRefused`]: The remote peer refused the connection.
	/// * [`ConnectionReset`]: The remote peer reset the connection.
	/// * [`DestUnreachable`]: The remote address is not reachable.
	/// * [`OutOfMemory`]: Insufficient memory available.
	/// * [`PeerAuth`]: Authentication or authorization failure.
	/// * [`Protocol`]: A protocol error occurred.
	///
	///
	/// [`Closed`]: enum.Error.html#variant.Closed
	/// [`ConnectionRefused`]: enum.Error.html#variant.ConnectionRefused
	/// [`ConnectionReset`]: enum.Error.html#variant.ConnectionReset
	/// [`DestUnreachable`]: enum.Error.html#variant.DestUnreachable
	/// [`OutOfMemory`]: enum.Error.html#variant.OutOfMemory
	/// [`PeerAuth`]: enum.Error.html#variant.PeerAuth
	/// [`Protocol`]: enum.Error.html#variant.Protocol
	pub fn start(self, nonblocking: bool) -> std::result::Result<Dialer, (Self, Error)>
	{
		let flags = if nonblocking { nng_sys::NNG_FLAG_NONBLOCK } else { 0 };

		// If there is an error starting the dialer, we don't want to consume
		// it. Instead, we'll return it to the user and they can decide what to
		// do.
		let rv = unsafe { nng_sys::nng_dialer_start(self.handle, flags as i32) };

		if let Some(e) = NonZeroU32::new(rv as u32) {
			Err((self, Error::from(e)))
		}
		else {
			let handle = Dialer { handle: self.handle };
			std::mem::forget(self);
			Ok(handle)
		}
	}
}

#[cfg(feature = "ffi-module")]
impl DialerOptions
{
	/// Returns the underlying `nng_dialer` object.
	pub fn nng_dialer(&self) -> nng_sys::nng_dialer { self.handle }
}

#[rustfmt::skip]
expose_options!{
	DialerOptions :: handle -> nng_sys::nng_dialer;

	GETOPT_BOOL = nng_sys::nng_dialer_get_bool;
	GETOPT_INT = nng_sys::nng_dialer_get_int;
	GETOPT_MS = nng_sys::nng_dialer_get_ms;
	GETOPT_SIZE = nng_sys::nng_dialer_get_size;
	GETOPT_SOCKADDR = nng_sys::nng_dialer_get_addr;
	GETOPT_STRING = nng_sys::nng_dialer_get_string;
	GETOPT_UINT64 = nng_sys::nng_dialer_get_uint64;

	SETOPT = nng_sys::nng_dialer_set;
	SETOPT_BOOL = nng_sys::nng_dialer_set_bool;
	SETOPT_INT = nng_sys::nng_dialer_set_int;
	SETOPT_MS = nng_sys::nng_dialer_set_ms;
	SETOPT_PTR = nng_sys::nng_dialer_set_ptr;
	SETOPT_SIZE = nng_sys::nng_dialer_set_size;
	SETOPT_STRING = nng_sys::nng_dialer_set_string;

	Gets -> [LocalAddr, Raw, ReconnectMinTime,
	         ReconnectMaxTime, RecvBufferSize,
	         RecvMaxSize, RecvTimeout,
	         SendBufferSize, SendTimeout,
	         SocketName, MaxTtl, Url,
	         protocol::reqrep::ResendTime,
	         protocol::survey::SurveyTime,
	         transport::tcp::NoDelay,
	         transport::tcp::KeepAlive,
	         transport::websocket::Protocol];
	Sets -> [ReconnectMinTime, ReconnectMaxTime,
	         RecvMaxSize, transport::tcp::NoDelay,
	         transport::tcp::KeepAlive,
	         transport::tls::CaFile,
	         transport::tls::CertKeyFile,
	         transport::websocket::RequestHeaders,
	         transport::websocket::Protocol];
}

impl Drop for DialerOptions
{
	fn drop(&mut self)
	{
		// Closing the dialer should only ever result in success or ECLOSED and
		// both of those mean that the drop was successful.
		let rv = unsafe { nng_sys::nng_dialer_close(self.handle) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED as i32,
			"Unexpected error code while closing dialer ({})",
			rv
		);
	}
}
