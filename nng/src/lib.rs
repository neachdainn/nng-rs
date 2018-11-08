//! A safe Rust wrapper for nanomsg-next-generation
extern crate nng_sys;

#[macro_use]
extern crate log;

use std::time::Duration;

#[macro_use]
mod macros;

mod error;
pub use error::{Error, ErrorKind, Result};

mod socket;
pub use socket::{Socket, Protocol};

mod dialer;
pub use dialer::{Dialer, DialerOptions};

mod listener;
pub use listener::{Listener, ListenerOptions};

mod addr;
pub use addr::SocketAddr;

mod message;
pub use message::{Message, Header, Body};

pub mod options;

/// A catch-all function for unsupported options operations.
unsafe extern "C" fn fake_opt<H, T>(_: H, _: *const std::os::raw::c_char, _: T) -> std::os::raw::c_int
{
	panic!("{} does not support the option operation on {}", stringify!(H), stringify!(T))
}

/// Converts a Rust Duration into an `nng_duration`.
fn duration_to_nng(dur: Option<Duration>) -> nng_sys::nng_duration
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

/// Converts an `nng_duration` into a Rust Duration.
fn nng_to_duration(ms: nng_sys::nng_duration) -> Option<Duration>
{
	if ms == nng_sys::NNG_DURATION_INFINITE {
		None
	} else if ms >= 0 {
		Some(Duration::from_millis(ms as u64))
	} else {
		panic!("Unexpected value for `nng_duration` ({})", ms)
	}
}
