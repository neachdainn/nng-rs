use std::{error, fmt, io};

use crate::message::Message;

/// Specialized `Result` type for use with nng.
pub type Result<T> = std::result::Result<T, Error>;

/// Specialized `Result` type for use with send operations.
pub(crate) type SendResult<T> = std::result::Result<T, SendError>;

/// Error type for send operations.
pub(crate) type SendError = (Message, Error);

/// Errors potentially returned by NNG operations.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[rustfmt::skip]
pub enum Error
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
	Protocol,

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
	SystemErr(u32),

	/// An unknown transport error occurred.
	TransportErr(u32),

	/// Unknown error code
	///
	/// Rather than panicking, we can just return this type. That will allow
	/// the user to continue operations normally if they so choose. It is also
	/// hidden from the docs because we do not really want to support this and
	/// to keep prevent additional error types from becoming breaking changes.
	#[doc(hidden)]
	Unknown(u32),
}
impl Error
{
	/// Converts an `i32` into an `Error`.
	///
	/// This is not an implementation of `From<i32>` because that would make
	/// the conversion a public part of this crate.
	#[rustfmt::skip]
	pub(crate) fn from_code(code: u32) -> Error
	{
		match code {
			0            => panic!("OK result passed as an error"),
			nng_sys::NNG_EINTR        => Error::Interrupted,
			nng_sys::NNG_ENOMEM       => Error::OutOfMemory,
			nng_sys::NNG_EINVAL       => Error::InvalidInput,
			nng_sys::NNG_EBUSY        => Error::Busy,
			nng_sys::NNG_ETIMEDOUT    => Error::TimedOut,
			nng_sys::NNG_ECONNREFUSED => Error::ConnectionRefused,
			nng_sys::NNG_ECLOSED      => Error::Closed,
			nng_sys::NNG_EAGAIN       => Error::TryAgain,
			nng_sys::NNG_ENOTSUP      => Error::NotSupported,
			nng_sys::NNG_EADDRINUSE   => Error::AddressInUse,
			nng_sys::NNG_ESTATE       => Error::IncorrectState,
			nng_sys::NNG_ENOENT       => Error::EntryNotFound,
			nng_sys::NNG_EPROTO       => Error::Protocol,
			nng_sys::NNG_EUNREACHABLE => Error::DestUnreachable,
			nng_sys::NNG_EADDRINVAL   => Error::AddressInvalid,
			nng_sys::NNG_EPERM        => Error::PermissionDenied,
			nng_sys::NNG_EMSGSIZE     => Error::MessageTooLarge,
			nng_sys::NNG_ECONNABORTED => Error::ConnectionAborted,
			nng_sys::NNG_ECONNRESET   => Error::ConnectionReset,
			nng_sys::NNG_ECANCELED    => Error::Canceled,
			nng_sys::NNG_ENOFILES     => Error::OutOfFiles,
			nng_sys::NNG_ENOSPC       => Error::OutOfSpace,
			nng_sys::NNG_EEXIST       => Error::ResourceExists,
			nng_sys::NNG_EREADONLY    => Error::ReadOnly,
			nng_sys::NNG_EWRITEONLY   => Error::WriteOnly,
			nng_sys::NNG_ECRYPTO      => Error::Crypto,
			nng_sys::NNG_EPEERAUTH    => Error::PeerAuth,
			nng_sys::NNG_ENOARG       => Error::NoArgument,
			nng_sys::NNG_EAMBIGUOUS   => Error::Ambiguous,
			nng_sys::NNG_EBADTYPE     => Error::BadType,
			nng_sys::NNG_EINTERNAL    => Error::Internal,
			c if c & nng_sys::NNG_ESYSERR != 0 => Error::SystemErr(c & !nng_sys::NNG_ESYSERR),
			c if c & nng_sys::NNG_ETRANERR != 0 => Error::TransportErr(c & !nng_sys::NNG_ETRANERR),
			_ => Error::Unknown(code),
		}
	}
}

impl From<SendError> for Error
{
	fn from((_, e): SendError) -> Error { e }
}

impl From<Error> for io::Error
{
	fn from(e: Error) -> io::Error
	{
		if let Error::SystemErr(c) = e {
			io::Error::from_raw_os_error(c as i32)
		}
		else {
			#[rustfmt::skip]
			#[allow(clippy::match_same_arms)]
			let new_kind = match e {
				Error::Interrupted => io::ErrorKind::Interrupted,
				Error::InvalidInput | Error::NoArgument => io::ErrorKind::InvalidInput,
				Error::TimedOut => io::ErrorKind::TimedOut,
				Error::TryAgain => io::ErrorKind::WouldBlock,
				Error::ConnectionRefused => io::ErrorKind::ConnectionRefused,
				Error::PermissionDenied => io::ErrorKind::PermissionDenied,
				Error::ConnectionAborted => io::ErrorKind::ConnectionAborted,
				Error::ConnectionReset => io::ErrorKind::ConnectionReset,
				Error::Canceled => io::ErrorKind::Interrupted, // I am not sure about this one
				Error::ResourceExists => io::ErrorKind::AlreadyExists,
				Error::BadType => io::ErrorKind::InvalidData,
				_ => io::ErrorKind::Other,
			};

			io::Error::new(new_kind, e)
		}
	}
}

impl error::Error for Error {}

impl fmt::Display for Error
{
	#[rustfmt::skip]
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		// Now, we could do a call into NNG for this but I think that adds
		// unnecessary complication since we would have to deal with c-strings
		// and unsafe code. We also couldn't do that for anything that wasn't a
		// "standard" error since that code is technically not thread-safe. It
		// really is just easier to hard-code the strings here.
		//
		// For the system error, we are going to lean on the standard library
		// to produce the output message for us. I am fairly certain that
		// creating one is not a heavy operation, so this should be fine.
		match *self {
			Error::Interrupted       => write!(f, "Interrupted"),
			Error::OutOfMemory       => write!(f, "Out of memory"),
			Error::InvalidInput      => write!(f, "Invalid argument"),
			Error::Busy              => write!(f, "Resource busy"),
			Error::TimedOut          => write!(f, "Timed out"),
			Error::ConnectionRefused => write!(f, "Connection refused"),
			Error::Closed            => write!(f, "Object closed"),
			Error::TryAgain          => write!(f, "Try again"),
			Error::NotSupported      => write!(f, "Not supported"),
			Error::AddressInUse      => write!(f, "Address in use"),
			Error::IncorrectState    => write!(f, "Incorrect state"),
			Error::EntryNotFound     => write!(f, "Entry not found"),
			Error::Protocol          => write!(f, "Protocol error"),
			Error::DestUnreachable   => write!(f, "Destination unreachable"),
			Error::AddressInvalid    => write!(f, "Address invalid"),
			Error::PermissionDenied  => write!(f, "Permission denied"),
			Error::MessageTooLarge   => write!(f, "Message too large"),
			Error::ConnectionReset   => write!(f, "Connection reset"),
			Error::ConnectionAborted => write!(f, "Connection aborted"),
			Error::Canceled          => write!(f, "Operation canceled"),
			Error::OutOfFiles        => write!(f, "Out of files"),
			Error::OutOfSpace        => write!(f, "Out of space"),
			Error::ResourceExists    => write!(f, "Resource already exists"),
			Error::ReadOnly          => write!(f, "Read only resource"),
			Error::WriteOnly         => write!(f, "Write only resource"),
			Error::Crypto            => write!(f, "Cryptographic error"),
			Error::PeerAuth          => write!(f, "Peer could not be authenticated"),
			Error::NoArgument        => write!(f, "Option requires argument"),
			Error::Ambiguous         => write!(f, "Ambiguous option"),
			Error::BadType           => write!(f, "Incorrect type"),
			Error::Internal          => write!(f, "Internal error detected"),
			Error::SystemErr(c)      => write!(f, "{}", io::Error::from_raw_os_error(c as i32)),
			Error::TransportErr(c)   => write!(f, "Transport error #{}", c),
			Error::Unknown(c)        => write!(f, "Unknown error code #{}", c),
		}
	}
}
