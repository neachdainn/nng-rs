//! A safe Rust wrapper for nanomsg-next-generation
extern crate nng_sys;

/// Converts a nng return code into a Rust `Result`
macro_rules! rv2res
{
	($rv:expr, $ok:expr) => (
		match $rv {
			0 => Ok($ok),
			e => Err($crate::Error::from_code(e)),
		}
	);

	($rv:expr) => ( rv2res!($rv, ()) )
}

mod error;
pub use error::{Error, Result};

mod socket;
pub use socket::Socket;
