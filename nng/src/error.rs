//! Error management module.
use nng_sys;

/// Specialized `Result` type for use with nng.
pub type Result<T> = ::std::result::Result<T, Error>;

/// The error type of nng operations.
pub struct Error
{
	
}
/*pub enum Error
{
	OK,
	Interrupted,
	OutOfMemory,
	InvalidArgument,
	ResourceBusy,
	TimedOut,
	ConnectionRefused,
	ObjectClosed,
	TryAgain,
	NotSupported,
	AddressInUse,
	IncorrectState,
	EntryNotFound,
	ProtocolError,
	DestinationUnreachable,
	AddressInvalid,
	PermissionDenied,
	MessageTooLarge,
	ConnectionReset,
	ConnectionAborted,
	OperationCanceled,
	OutOfFiles,
	OutOfSpace,
	ResourceExists,
	ReadOnlyResource,
	WriteOnlyResource,
	Crypto,
	Authentication,
	Argument,
	Ambiguous,
	IncorrectType,
	InternalError,
	SystemErr(i32),
	TransportErr(i32),
	Unknown(i32),
}
impl Error
{
	/// Converts a nng error code into an Error type.
	pub(crate) fn from_code(code: i32) -> Self
	{
		// Normally one would use `From<i32>` as that would allow for the `?`
		// operator to be used. However, I don't like the trait implementation
		// being usable outside of the crate and we already have to use a macro
		// for the error conversion anyway. Might as well make it a regular,
		// private fuction.
		if code == nng_sys::nng_errno_enum::NNG_EINTR as i32 {
			Error::Interrupted
		} else {
			Error::Unknown(code)
		}
		/*NNG_EINTR        = 1,
		NNG_ENOMEM       = 2,
		NNG_EINVAL       = 3,
		NNG_EBUSY        = 4,
		NNG_ETIMEDOUT    = 5,
		NNG_ECONNREFUSED = 6,
		NNG_ECLOSED      = 7,
		NNG_EAGAIN       = 8,
		NNG_ENOTSUP      = 9,
		NNG_EADDRINUSE   = 10,
		NNG_ESTATE       = 11,
		NNG_ENOENT       = 12,
		NNG_EPROTO       = 13,
		NNG_EUNREACHABLE = 14,
		NNG_EADDRINVAL   = 15,
		NNG_EPERM        = 16,
		NNG_EMSGSIZE     = 17,
		NNG_ECONNABORTED = 18,
		NNG_ECONNRESET   = 19,
		NNG_ECANCELED    = 20,
		NNG_ENOFILES     = 21,
		NNG_ENOSPC       = 22,
		NNG_EEXIST       = 23,
		NNG_EREADONLY    = 24,
		NNG_EWRITEONLY   = 25,
		NNG_ECRYPTO      = 26,
		NNG_EPEERAUTH    = 27,
		NNG_ENOARG       = 28,
		NNG_EAMBIGUOUS   = 29,
		NNG_EBADTYPE     = 30,
		NNG_EINTERNAL    = 1000,
		NNG_ESYSERR      = 0x10000000,
		NNG_ETRANERR = 0x20000000*/
	}
}*/
