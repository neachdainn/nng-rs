//! Implementation details about implementing options.
use std::os::raw::{c_char, c_int};
use std::time::Duration;
use crate::error::Result;
use crate::addr::SocketAddr;

/// Marks a type as an `nng` option.
///
/// This trait does not enforce the availability of any specific options on
/// any specific type. Calling these methods incorrectly will result in `nng`
/// returning an error code.
pub trait Opt
{
	/// The Rust type that the option should (eventually) return.
	type OptType;

	/// Get the value of the option using the specified type.
	fn get<T: HasOpts>(s: &T) -> Result<Self::OptType>;

	/// Set the value of the option using the specified type.
	fn set<T: HasOpts>(s: &T, val: Self::OptType) -> Result<()>;
}

/// Marks a type that can get and set `nng` options.
pub trait HasOpts: Sized
{
	/// Underlying `nng` type.
	type Handle;

	/// Raw `nng` function for getting a boolean option.
	const GETOPT_BOOL: unsafe extern "C" fn(Self::Handle, *const c_char, *mut bool) -> c_int;
	/// Raw `nng` function to get an `nng_duration`.
	const GETOPT_MS: unsafe extern "C" fn(Self::Handle, *const c_char, *mut nng_sys::nng_duration) -> c_int;
	/// Raw `nng` function for getting a `size_t` option.
	const GETOPT_SIZE: unsafe extern "C" fn(Self::Handle, *const c_char, *mut usize) -> c_int;
	/// Raw `nng` function for getting an `nng_sockaddr` option.
	const GETOPT_SOCKADDR: unsafe extern "C" fn(Self::Handle, *const c_char, *mut nng_sys::nng_sockaddr) -> c_int;

	/// Raw `nng` function to set an `nng_duration`.
	const SETOPT_MS: unsafe extern "C" fn(Self::Handle, *const c_char, nng_sys::nng_duration) -> c_int;

	/// Returns the underlying `nng` type.
	fn handle(&self) -> Self::Handle;

	/// Get the boolean option.
	fn getopt_bool(&self, opt: *const c_char) -> Result<bool>
	{
		let mut raw = false;
		let rv = unsafe {
			(Self::GETOPT_BOOL)(self.handle(), opt, &mut raw as _)
		};

		rv2res!(rv, raw)
	}

	/// Get the duration from the option.
	fn getopt_ms(&self, opt: *const c_char) -> Result<Option<Duration>>
	{
		let mut dur: nng_sys::nng_duration = 0;
		let rv = unsafe {
			(Self::GETOPT_MS)(self.handle(), opt, &mut dur as _)
		};

		rv2res!(rv, {
			if dur == nng_sys::NNG_DURATION_INFINITE {
				None
			} else if dur >= 0 {
				Some(Duration::from_millis(dur as u64))
			} else {
				panic!("Unexpected value for `nng_duration` ({})", dur)
			}
		})
	}

	/// Get the `size_t` option.
	fn getopt_size(&self, opt: *const c_char) -> Result<usize>
	{
		let mut sz = 0;
		let rv = unsafe {
			(Self::GETOPT_SIZE)(self.handle(), opt, &mut sz as _)
		};

		rv2res!(rv, sz)
	}

	/// Get the specified socket address option.
	fn getopt_sockaddr(&self, opt: *const c_char) -> Result<SocketAddr>
	{
		unsafe {
			let mut addr: nng_sys::nng_sockaddr = std::mem::uninitialized();
			let rv = (Self::GETOPT_SOCKADDR)(self.handle(), opt, &mut addr as _);

			rv2res!(rv, addr.into())
		}
	}

	/// Set the duration to the option.
	fn setopt_ms(&self, opt: *const c_char, dur: Option<Duration>) -> Result<()>
	{
		// The subsecond milliseconds is guaranteed to be less than 1000, which
		// means converting from `u32` to `i32` is safe. The only other
		// potential issue is converting the `u64` of seconds to an `i32`.
		use std::i32::MAX;

		let ms = match dur {
			None => nng_sys::NNG_DURATION_INFINITE,
			Some(d) => {
				let secs = if d.as_secs() > MAX as u64 { MAX } else { d.as_secs() as i32 };
				let millis = d.subsec_millis() as i32;

				secs.saturating_mul(1000).saturating_add(millis)
			}
		};

		let rv = unsafe {
			(Self::SETOPT_MS)(self.handle(), opt, ms)
		};
		rv2res!(rv)
	}
}
