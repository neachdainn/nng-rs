use crate::dialer::Dialer;
use crate::listener::Listener;

/// A nanomsg-next-generation pipe.
///
/// A pipe can be thought of as a single connection and are associated with either the listener or
/// dialer that created them. Therefore, they are automatically associated with a single socket.
///
/// Most applications should never concern themselves with individual pipes. However, it is possible
/// to access a piope when more information about the source of the message is needed or when more
/// control is required over message delivery.
///
/// See the [nng documentation][1] for more information.
///
/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_pipe.5
#[derive(Debug, PartialEq, Eq)]
pub struct Pipe
{
	/// The underlying nng pipe.
	handle: nng_sys::nng_pipe,
}
impl Pipe
{
	/// Returns the dialer associated with this pipe, if any.
	pub fn dialer(&self) -> Option<Dialer>
	{
		let dialer = unsafe { nng_sys::nng_pipe_dialer(self.handle) };

		if dialer.id > 0 {
			Some(Dialer::from_nng_sys(dialer))
		} else { None }
	}

	/// Returns the listener associated with this pipe, if any.
	pub fn listener(&self) -> Option<Listener>
	{
	
		let listener = unsafe { nng_sys::nng_pipe_listener(self.handle) };

		if listener.id > 0 {
			Some(Listener::from_nng_sys(listener))
		} else { None }
	}

	/// Returns the ID of the owning socket.
	///
	/// This function should be considered unstable. Eventually it should be possible to get the
	/// socket itself, rather than just the ID.
	pub fn socket_id(&self) -> i32
	{
		let socket = unsafe { nng_sys::nng_pipe_socket(self.handle) };
		assert!(socket.id > 0, "Invalid socket associated with valid pipe");

		unsafe { nng_sys::nng_socket_id(socket) }
	}

	/// Closes the pipe.
	///
	/// Messages that have been submitted for sending may be flushed or delivered, depending upon
	/// the transport and the linger option. Pipe are automatically closed when their creator closes
	/// or when the remote peer closes the underlying connection.
	pub fn close(self)
	{
		// The pipe either closes succesfully, was already closed, or was never open. In any of
		// those scenarios, the pipe is in the desired state. As such, we don't care about the
		// return value.
		let rv = unsafe { nng_sys::nng_pipe_close(self.handle) };
		assert!(
			rv == 0 || rv == nng_sys::NNG_ECLOSED,
			"Unexpected error code while closing pipe ({})", rv
		);
	}

	/// Returns the positive identifier for the pipe.
	pub fn id(&self) -> i32
	{
		let id = unsafe { nng_sys::nng_pipe_id(self.handle) };
		assert!(id > 0, "Invalid pipe ID returned from valid pipe");

		id
	}

	/// Returns the underlying nng handle for the pipe.
	pub(crate) fn handle(&self) -> nng_sys::nng_pipe
	{
		self.handle
	}

	/// Create a new Pipe handle from a libnng handle.
	///
	/// This function will panic if the handle is not valid.
	pub(crate) fn from_nng_sys(handle: nng_sys::nng_pipe) -> Self
	{
		assert!(handle.id > 0, "Pipe handle is not initialized");
		Pipe { handle }
	}
}

expose_options!{
	Pipe :: handle -> nng_sys::nng_pipe;

	GETOPT_BOOL = nng_sys::nng_pipe_getopt_bool;
	GETOPT_INT = nng_sys::nng_pipe_getopt_int;
	GETOPT_MS = nng_sys::nng_pipe_getopt_ms;
	GETOPT_SIZE = nng_sys::nng_pipe_getopt_size;
	GETOPT_SOCKADDR = nng_sys::nng_pipe_getopt_sockaddr;
	GETOPT_STRING = nng_sys::nng_pipe_getopt_string;

	SETOPT = crate::fake_genopt;
	SETOPT_BOOL = crate::fake_opt;
	SETOPT_INT = crate::fake_opt;
	SETOPT_MS = crate::fake_opt;
	SETOPT_SIZE = crate::fake_opt;
	SETOPT_STRING =crate::fake_opt;

	Gets -> [LocalAddr, RemAddr, RecvMaxSize,
	         transport::tcp::NoDelay,
	         transport::tcp::KeepAlive,
	         transport::tls::TlsVerified,
	         transport::websocket::RequestHeaders,
	         transport::websocket::ResponseHeaders];
	Sets -> [];
}
