use std::num::NonZeroU32;

use crate::{
	error::{Error, Result},
	socket::RawSocket,
};

/// Forwards messages from socket _s1_ to socket _s2_ and vice versa.
///
/// This function is used to create forwarders, which can be used to create
/// complex network topologies to provide for improved horizontal scalability,
/// reliability, and isolation. The provided sockets must have protocols that
/// are compatible with each other. For example, if _s1_ is a _sub_ socket then
/// _s2_ must be a _pub_ socket, or if _s1_ is a _bus_ socket then _s2_ must be
/// a _bus_ socket as well.
///
/// Note that some protocols have a maximum time-to-live to protect against
/// forwarding loops and especially amplification loops. In these cases, the
/// default limit (usually 8), ensures that messages will self-terminate when
/// they have passed through too many forwarders, protecting the network from
/// unlimited message amplification that can arise through misconfiguration.
/// This is controlled by the `MaxTtl` option.
///
/// This function does not return unless one of the sockets encounters an
/// error or is closed. For more information see the [NNG documentation][1].
///
/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_device.3
pub fn forwarder(s1: RawSocket, s2: RawSocket) -> Result<()>
{
	let rv = unsafe { nng_sys::nng_device(s1.socket.handle(), s2.socket.handle()) };

	// Appease Clippy.
	drop(s1);
	drop(s2);

	if let Some(e) = NonZeroU32::new(rv as u32) {
		Err(Error::from(e))
	}
	else {
		unreachable!("nng_device returned with no errror");
	}
}

/// Reflects a socket's sent messages back at itself.
///
/// The provided socket must have a protocol that is bidirectional and can peer
/// with itself, such as a _pair_ or _bus_ socket. A reflector or loop-back
/// device is created where valid messages from the socket are simply returned
/// back to the sender.
///
/// This function does not return unless the socket encounters an error or is
/// closed. For more information, see the [NNG documentation][1].
///
/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_device.3
pub fn reflector(s1: RawSocket) -> Result<()>
{
	let rv = unsafe {
		nng_sys::nng_device(s1.socket.handle(), nng_sys::nng_socket::NNG_SOCKET_INITIALIZER)
	};

	drop(s1); // Appease Clippy

	if let Some(e) = NonZeroU32::new(rv as u32) {
		Err(Error::from(e))
	}
	else {
		unreachable!("nng_device returned with no errror");
	}
}
