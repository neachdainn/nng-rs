//! A safe Rust wrapper for nanomsg-next-generation
extern crate nng_sys;

use std::time::Duration;

/// Converts a nng return code into a Rust `Result`
macro_rules! rv2res
{
	($rv:expr, $ok:expr) => (
		match $rv {
			0 => Ok($ok),
			e => Err($crate::error::ErrorKind::from_code(e).into()),
		}
	);

	($rv:expr) => ( rv2res!($rv, ()) )
}

mod error;
pub use error::{Error, ErrorKind, Result};

mod socket;
pub use socket::Socket;

pub mod dialer;
pub mod listener;

mod addr;
pub use addr::SocketAddr;

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
