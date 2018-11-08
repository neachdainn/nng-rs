use std::{error, fmt, io};
use crate::message::Message;

/// Specialized `Result` type for use with nng.
pub type Result<T> = std::result::Result<T, Error>;

/// Specialized `Result` type for use with send operations.
pub(crate) type SendResult<T> = std::result::Result<T, SendError>;

/// Error type for send operations.
pub(crate) type SendError = (Message, Error);

/// The error type of nng operations.
#[derive(Debug)]
pub struct Error
{
	/// The underlying nng error code
	kind: ErrorKind,
}
impl Error
{
	/// Returns the underlying `ErrorKind`.
	pub fn kind(&self) -> ErrorKind
	{
		self.kind
	}
}

impl error::Error for Error {}

impl From<ErrorKind> for Error
{
	fn from(kind: ErrorKind) -> Error
	{
		Error { kind }
	}
}

impl From<SendError> for Error
{
	fn from((_, e): SendError) -> Error
	{
		e
	}
}

impl From<Error> for io::Error
{
	fn from(e: Error) -> io::Error
	{
		if let ErrorKind::SystemErr(c) = e.kind {
			io::Error::from_raw_os_error(c)
		} else {
			let new_kind = match e.kind {
				ErrorKind::Interrupted => io::ErrorKind::Interrupted,
				ErrorKind::InvalidInput | ErrorKind::NoArgument => io::ErrorKind::InvalidInput,
				ErrorKind::TimedOut => io::ErrorKind::TimedOut,
				ErrorKind::TryAgain => io::ErrorKind::WouldBlock,
				ErrorKind::ConnectionRefused => io::ErrorKind::ConnectionRefused,
				ErrorKind::PermissionDenied => io::ErrorKind::PermissionDenied,
				ErrorKind::ConnectionAborted => io::ErrorKind::ConnectionAborted,
				ErrorKind::ConnectionReset => io::ErrorKind::ConnectionReset,
				ErrorKind::Canceled => io::ErrorKind::Interrupted, // I am not sure about this one
				ErrorKind::ResourceExists => io::ErrorKind::AlreadyExists,
				ErrorKind::BadType => io::ErrorKind::InvalidData,
				_ => io::ErrorKind::Other,
			};

			io::Error::new(new_kind, e)
		}
	}
}

impl fmt::Display for Error
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		write!(f, "{}", self.kind)
	}
}

/// General categories of nng errors
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind
{
	/// The operation was interrupted
	Interrupted,

	/// Insufficient memory available to perform the operation
	OutOfMemory,

	/// An invalid argument was specified
	InvalidInput,

	/// The resource is busy
	Busy,

	/// The operation timed out
	TimedOut,

	/// Connection refused by peer
	ConnectionRefused,

	/// The resource is already closed or was never opened
	Closed,

	/// Operation would block
	TryAgain,

	/// Operation is not supported by the library
	NotSupported,

	/// The address is already in use
	AddressInUse,

	/// The resource is not in the appropriate state for the operation
	IncorrectState,

	/// Entry was not found
	EntryNotFound,

	/// A protocol error occurred
	ProtocolError,

	/// Remote address is unreachable
	DestUnreachable,

	/// An invalid URL was specified
	AddressInvalid,

	/// Did not have the required permissions to complete the operation
	PermissionDenied,

	/// The message was too large
	MessageTooLarge,

	/// Connection attempt aborted
	ConnectionAborted,

	/// Connection reset or closed by peer
	ConnectionReset,

	/// The operation was canceled
	Canceled,

	/// Out of files
	OutOfFiles,

	/// Insufficient persistent storage
	OutOfSpace,

	/// Resource already exists
	ResourceExists,

	/// The specified option is read-only
	ReadOnly,

	/// The specified option is write-only
	WriteOnly,

	/// A cryptographic error occurred
	Crypto,

	/// Authentication or authorization failure
	PeerAuth,

	/// The option requires an argument but it was not present
	NoArgument,

	/// Parsed option matches more than one specification
	Ambiguous,

	/// Incorrect type used for option
	BadType,

	/// An internal error occurred.
	Internal,

	/// An unknown system error occurred.
	SystemErr(i32),

	/// An unknown transport error occurred.
	TransportErr(i32),

	/// Unknown error code
	///
	/// Rather than panicking, we can just return this type. That will allow
	/// the user to continue operations normally if they so choose. It is also
	/// hidden from the docs because we do not really want to support this and
	/// to keep prevent additional error types from becoming breaking changes.
	#[doc(hidden)]
	Unknown(i32),
}
impl ErrorKind
{
	/// Converts an `i32` into an `ErrorKind`.
	///
	/// This is not an implementation of `From<i32>` because that would make
	/// the conversion a public part of this crate.
	pub(crate) fn from_code(code: i32) -> ErrorKind
	{
		match code {
			0            => panic!("OK result passed as an error"),
			nng_sys::NNG_EINTR        => ErrorKind::Interrupted,
			nng_sys::NNG_ENOMEM       => ErrorKind::OutOfMemory,
			nng_sys::NNG_EINVAL       => ErrorKind::InvalidInput,
			nng_sys::NNG_EBUSY        => ErrorKind::Busy,
			nng_sys::NNG_ETIMEDOUT    => ErrorKind::TimedOut,
			nng_sys::NNG_ECONNREFUSED => ErrorKind::ConnectionRefused,
			nng_sys::NNG_ECLOSED      => ErrorKind::Closed,
			nng_sys::NNG_EAGAIN       => ErrorKind::TryAgain,
			nng_sys::NNG_ENOTSUP      => ErrorKind::NotSupported,
			nng_sys::NNG_EADDRINUSE   => ErrorKind::AddressInUse,
			nng_sys::NNG_ESTATE       => ErrorKind::IncorrectState,
			nng_sys::NNG_ENOENT       => ErrorKind::EntryNotFound,
			nng_sys::NNG_EPROTO       => ErrorKind::ProtocolError,
			nng_sys::NNG_EUNREACHABLE => ErrorKind::DestUnreachable,
			nng_sys::NNG_EADDRINVAL   => ErrorKind::AddressInvalid,
			nng_sys::NNG_EPERM        => ErrorKind::PermissionDenied,
			nng_sys::NNG_EMSGSIZE     => ErrorKind::MessageTooLarge,
			nng_sys::NNG_ECONNABORTED => ErrorKind::ConnectionAborted,
			nng_sys::NNG_ECONNRESET   => ErrorKind::ConnectionReset,
			nng_sys::NNG_ECANCELED    => ErrorKind::Canceled,
			nng_sys::NNG_ENOFILES     => ErrorKind::OutOfFiles,
			nng_sys::NNG_ENOSPC       => ErrorKind::OutOfSpace,
			nng_sys::NNG_EEXIST       => ErrorKind::ResourceExists,
			nng_sys::NNG_EREADONLY    => ErrorKind::ReadOnly,
			nng_sys::NNG_EWRITEONLY   => ErrorKind::WriteOnly,
			nng_sys::NNG_ECRYPTO      => ErrorKind::Crypto,
			nng_sys::NNG_EPEERAUTH    => ErrorKind::PeerAuth,
			nng_sys::NNG_ENOARG       => ErrorKind::NoArgument,
			nng_sys::NNG_EAMBIGUOUS   => ErrorKind::Ambiguous,
			nng_sys::NNG_EBADTYPE     => ErrorKind::BadType,
			nng_sys::NNG_EINTERNAL    => ErrorKind::Internal,
			c if c & nng_sys::NNG_ESYSERR != 0 => ErrorKind::SystemErr(c & !nng_sys::NNG_ESYSERR),
			c if c & nng_sys::NNG_ETRANERR != 0 => ErrorKind::TransportErr(c & !nng_sys::NNG_ETRANERR),
			_ => ErrorKind::Unknown(code),
		}
	}
}

impl fmt::Display for ErrorKind
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		// Now, we could do a call into nng for this but I think that adds
		// unnecessary complication since we would have to deal with c-strings
		// and unsafe code. We also couldn't do that for anything that wasn't a
		// "standard" error since that code is technically not thread-safe. It
		// really is just easier to hard-code the strings here.
		//
		// For the system error, we are going to lean on the standard library
		// to produce the output message for us. I am fairly certain that
		// creating one is not a heavy operation, so this should be fine.
		match *self {
			ErrorKind::Interrupted       => write!(f, "Interrupted"),
			ErrorKind::OutOfMemory       => write!(f, "Out of memory"),
			ErrorKind::InvalidInput      => write!(f, "Invalid argument"),
			ErrorKind::Busy              => write!(f, "Resource busy"),
			ErrorKind::TimedOut          => write!(f, "Timed out"),
			ErrorKind::ConnectionRefused => write!(f, "Connection refused"),
			ErrorKind::Closed            => write!(f, "Object closed"),
			ErrorKind::TryAgain          => write!(f, "Try again"),
			ErrorKind::NotSupported      => write!(f, "Not supported"),
			ErrorKind::AddressInUse      => write!(f, "Address in use"),
			ErrorKind::IncorrectState    => write!(f, "Incorrect state"),
			ErrorKind::EntryNotFound     => write!(f, "Entry not found"),
			ErrorKind::ProtocolError     => write!(f, "Protocol error"),
			ErrorKind::DestUnreachable   => write!(f, "Destination unreachable"),
			ErrorKind::AddressInvalid    => write!(f, "Address invalid"),
			ErrorKind::PermissionDenied  => write!(f, "Permission denied"),
			ErrorKind::MessageTooLarge   => write!(f, "Message too large"),
			ErrorKind::ConnectionReset   => write!(f, "Connection reset"),
			ErrorKind::ConnectionAborted => write!(f, "Connection aborted"),
			ErrorKind::Canceled          => write!(f, "Operation canceled"),
			ErrorKind::OutOfFiles        => write!(f, "Out of files"),
			ErrorKind::OutOfSpace        => write!(f, "Out of space"),
			ErrorKind::ResourceExists    => write!(f, "Resource already exists"),
			ErrorKind::ReadOnly          => write!(f, "Read only resource"),
			ErrorKind::WriteOnly         => write!(f, "Write only resource"),
			ErrorKind::Crypto            => write!(f, "Cryptographic error"),
			ErrorKind::PeerAuth          => write!(f, "Peer could not be authenticated"),
			ErrorKind::NoArgument        => write!(f, "Option requires argument"),
			ErrorKind::Ambiguous         => write!(f, "Ambiguous option"),
			ErrorKind::BadType           => write!(f, "Incorrect type"),
			ErrorKind::Internal          => write!(f, "Internal error detected"),
			ErrorKind::SystemErr(c)      => write!(f, "{}", io::Error::from_raw_os_error(c)),
			ErrorKind::TransportErr(c)   => write!(f, "Transport error #{}", c),
			ErrorKind::Unknown(c)        => write!(f, "Unknown error code #{}", c),
		}
	}
}
