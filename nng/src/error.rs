//! Error management module.
use std::{error, fmt, io};

/// Specialized `Result` type for use with nng.
pub type Result<T> = ::std::result::Result<T, Error>;

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
			codes::EINTR        => ErrorKind::Interrupted,
			codes::ENOMEM       => ErrorKind::OutOfMemory,
			codes::EINVAL       => ErrorKind::InvalidInput,
			codes::EBUSY        => ErrorKind::Busy,
			codes::ETIMEDOUT    => ErrorKind::TimedOut,
			codes::ECONNREFUSED => ErrorKind::ConnectionRefused,
			codes::ECLOSED      => ErrorKind::Closed,
			codes::EAGAIN       => ErrorKind::TryAgain,
			codes::ENOTSUP      => ErrorKind::NotSupported,
			codes::EADDRINUSE   => ErrorKind::AddressInUse,
			codes::ESTATE       => ErrorKind::IncorrectState,
			codes::ENOENT       => ErrorKind::EntryNotFound,
			codes::EPROTO       => ErrorKind::ProtocolError,
			codes::EUNREACHABLE => ErrorKind::DestUnreachable,
			codes::EADDRINVAL   => ErrorKind::AddressInvalid,
			codes::EPERM        => ErrorKind::PermissionDenied,
			codes::EMSGSIZE     => ErrorKind::MessageTooLarge,
			codes::ECONNABORTED => ErrorKind::ConnectionAborted,
			codes::ECONNRESET   => ErrorKind::ConnectionReset,
			codes::ECANCELED    => ErrorKind::Canceled,
			codes::ENOFILES     => ErrorKind::OutOfFiles,
			codes::ENOSPC       => ErrorKind::OutOfSpace,
			codes::EEXIST       => ErrorKind::ResourceExists,
			codes::EREADONLY    => ErrorKind::ReadOnly,
			codes::EWRITEONLY   => ErrorKind::WriteOnly,
			codes::ECRYPTO      => ErrorKind::Crypto,
			codes::EPEERAUTH    => ErrorKind::PeerAuth,
			codes::ENOARG       => ErrorKind::NoArgument,
			codes::EAMBIGUOUS   => ErrorKind::Ambiguous,
			codes::EBADTYPE     => ErrorKind::BadType,
			codes::EINTERNAL    => ErrorKind::Internal,
			c if c & codes::ESYSERR != 0 => ErrorKind::SystemErr(c & !codes::ESYSERR),
			c if c & codes::ETRANERR != 0 => ErrorKind::TransportErr(c & !codes::ETRANERR),
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

/// Mapping from the `nng-sys` enum into constants.
///
/// We don't do this in the sys crate because:
///
/// 1. I think that is messier.
/// 2. The error codes really are an enum rather than just a list of constants.
mod codes
{
	use nng_sys;

	pub const EINTR:        i32 = nng_sys::nng_errno_enum::NNG_EINTR        as i32;
	pub const ENOMEM:       i32 = nng_sys::nng_errno_enum::NNG_ENOMEM       as i32;
	pub const EINVAL:       i32 = nng_sys::nng_errno_enum::NNG_EINVAL       as i32;
	pub const EBUSY:        i32 = nng_sys::nng_errno_enum::NNG_EBUSY        as i32;
	pub const ETIMEDOUT:    i32 = nng_sys::nng_errno_enum::NNG_ETIMEDOUT    as i32;
	pub const ECONNREFUSED: i32 = nng_sys::nng_errno_enum::NNG_ECONNREFUSED as i32;
	pub const ECLOSED:      i32 = nng_sys::nng_errno_enum::NNG_ECLOSED      as i32;
	pub const EAGAIN:       i32 = nng_sys::nng_errno_enum::NNG_EAGAIN       as i32;
	pub const ENOTSUP:      i32 = nng_sys::nng_errno_enum::NNG_ENOTSUP      as i32;
	pub const EADDRINUSE:   i32 = nng_sys::nng_errno_enum::NNG_EADDRINUSE   as i32;
	pub const ESTATE:       i32 = nng_sys::nng_errno_enum::NNG_ESTATE       as i32;
	pub const ENOENT:       i32 = nng_sys::nng_errno_enum::NNG_ENOENT       as i32;
	pub const EPROTO:       i32 = nng_sys::nng_errno_enum::NNG_EPROTO       as i32;
	pub const EUNREACHABLE: i32 = nng_sys::nng_errno_enum::NNG_EUNREACHABLE as i32;
	pub const EADDRINVAL:   i32 = nng_sys::nng_errno_enum::NNG_EADDRINVAL   as i32;
	pub const EPERM:        i32 = nng_sys::nng_errno_enum::NNG_EPERM        as i32;
	pub const EMSGSIZE:     i32 = nng_sys::nng_errno_enum::NNG_EMSGSIZE     as i32;
	pub const ECONNABORTED: i32 = nng_sys::nng_errno_enum::NNG_ECONNABORTED as i32;
	pub const ECONNRESET:   i32 = nng_sys::nng_errno_enum::NNG_ECONNRESET   as i32;
	pub const ECANCELED:    i32 = nng_sys::nng_errno_enum::NNG_ECANCELED    as i32;
	pub const ENOFILES:     i32 = nng_sys::nng_errno_enum::NNG_ENOFILES     as i32;
	pub const ENOSPC:       i32 = nng_sys::nng_errno_enum::NNG_ENOSPC       as i32;
	pub const EEXIST:       i32 = nng_sys::nng_errno_enum::NNG_EEXIST       as i32;
	pub const EREADONLY:    i32 = nng_sys::nng_errno_enum::NNG_EREADONLY    as i32;
	pub const EWRITEONLY:   i32 = nng_sys::nng_errno_enum::NNG_EWRITEONLY   as i32;
	pub const ECRYPTO:      i32 = nng_sys::nng_errno_enum::NNG_ECRYPTO      as i32;
	pub const EPEERAUTH:    i32 = nng_sys::nng_errno_enum::NNG_EPEERAUTH    as i32;
	pub const ENOARG:       i32 = nng_sys::nng_errno_enum::NNG_ENOARG       as i32;
	pub const EAMBIGUOUS:   i32 = nng_sys::nng_errno_enum::NNG_EAMBIGUOUS   as i32;
	pub const EBADTYPE:     i32 = nng_sys::nng_errno_enum::NNG_EBADTYPE     as i32;
	pub const EINTERNAL:    i32 = nng_sys::nng_errno_enum::NNG_EINTERNAL    as i32;
	pub const ESYSERR:      i32 = nng_sys::nng_errno_enum::NNG_ESYSERR      as i32;
	pub const ETRANERR:     i32 = nng_sys::nng_errno_enum::NNG_ETRANERR     as i32;
}
