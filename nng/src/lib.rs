//! A safe Rust wrapper for nanomsg-next-generation
extern crate nng_sys;

use std::time::Duration;

/// Converts a `nng` return code into a Rust `Result`.
macro_rules! rv2res
{
	($rv:expr, $ok:expr) => (
		match $rv {
			0 => Ok($ok),
			e => Err(crate::error::ErrorKind::from_code(e).into()),
		}
	);

	($rv:expr) => ( rv2res!($rv, ()) )
}

/// Checks an `nng` return code and validates the pointer.
macro_rules! validate_ptr
{
	($rv:ident, $ptr:ident) => (
		if $rv != 0 {
			return Err(crate::error::ErrorKind::from_code($rv).into());
		}
		assert!($ptr != std::ptr::null_mut(), "Nng returned a null pointer from a successful function");
	)
}

/// Utility macro for creating a new option type.
///
/// This is 90% me just playing around with macros. It is probably a terrible
/// way to go around doing this task but, then again, this whole options
/// business has been a complete mess.
macro_rules! create_option
{
	(
		$(#[$attr:meta])*
		$opt:ident -> $ot:ty:
		Get $g:ident = $gexpr:stmt;
		Set $s:ident $v:ident = $sexpr:stmt;
	) => {
		$(#[$attr])*
		pub enum $opt {}
		impl $crate::options::private::Opt for $opt
		{
			type OptType = $ot;

			fn get<T: $crate::options::private::HasOpts>($g: &T) -> $crate::error::Result<Self::OptType> { $gexpr }
			fn set<T: $crate::options::private::HasOpts>($s: &T, $v: Self::OptType) -> $crate::error::Result<()> { $sexpr }
		}
	}
}

/// Implements the specified options for the type.
macro_rules! expose_options
{
	(
		$struct:ident :: $member:ident -> $handle:ty;
		GETOPT_BOOL = $go_b:expr;
		GETOPT_MS = $go_ms:expr;
		GETOPT_SIZE = $go_sz:expr;
		GETOPT_SOCKADDR = $go_sa:expr;

		SETOPT_MS = $so_ms:expr;

		Gets -> [$($getters:ident),*];
		Sets -> [$($setters:ident),*];
	) => {
		impl $crate::options::private::HasOpts for $struct
		{
			type Handle = $handle;
			fn handle(&self) -> Self::Handle { self.$member }

			const GETOPT_BOOL: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut bool) -> std::os::raw::c_int = $go_b;
			const GETOPT_MS: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut nng_sys::nng_duration) -> std::os::raw::c_int = $go_ms;
			const GETOPT_SIZE: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut usize) -> std::os::raw::c_int = $go_sz;
			const GETOPT_SOCKADDR: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut nng_sys::nng_sockaddr) -> std::os::raw::c_int = $go_sa;

			const SETOPT_MS: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, nng_sys::nng_duration) -> std::os::raw::c_int = $so_ms;
		}

		$(impl $crate::options::GetOpt<$crate::options::$getters> for $struct {})*
		$(impl $crate::options::SetOpt<$crate::options::$setters> for $struct {})*
	}
}

mod error;
pub use error::{Error, ErrorKind, Result};

mod socket;
pub use socket::{Socket, Protocol};

pub mod dialer;
pub mod listener;

mod addr;
pub use addr::SocketAddr;

pub mod message;
pub mod options;

/// Converts a `Duration` into an `nng_duration`.
///
/// This function is saturating in that if the supplied duration is longer than
/// can be represented then it will return the max possible duration. A value
/// of `None` converts to the `nng` representation of infinite.
fn duration_to_nng(dur: Option<Duration>) -> i32
{
	// The subsecond milliseconds is guaranteed to be less than 1000, which
	// means converting from `u32` to `i32` is safe. The only other
	// potential issue is converting the `u64` of seconds to an `i32`.
	use std::i32::MAX;

	match dur {
		None => nng_sys::NNG_DURATION_INFINITE,
		Some(d) => {
			let secs = if d.as_secs() > MAX as u64 { MAX } else { d.as_secs() as i32 };
			let millis = d.subsec_millis() as i32;

			secs.saturating_mul(1000).saturating_add(millis)
		}
	}

}

/// Converts an `nng_duration` into a Rust `Duration`.
///
/// This function assumes that the only special duration value is the `nng`
/// representation of inifinite. Any other negative value is an error that will
/// cause a panic. An infinite value translates to `None`.
fn nng_to_duration(dur: i32) -> Option<Duration>
{
	if dur == nng_sys::NNG_DURATION_INFINITE {
		None
	} else if dur >= 0 {
		Some(Duration::from_millis(dur as u64))
	} else {
		panic!("Unexpected value for `nng_duration` ({})", dur)
	}
}
