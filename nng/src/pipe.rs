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
}
