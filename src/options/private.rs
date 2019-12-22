//! Implementation details about implementing options.
use std::{
	ffi::{CStr, CString},
	mem::MaybeUninit,
	os::raw::{c_char, c_int, c_void},
	ptr,
	time::Duration,
};

use crate::{
	addr::SocketAddr,
	error::{Error, Result},
	util::validate_ptr,
};

/// Exposes the ability to get and set the option.
///
/// This trait does not enforce the availability of any specific options on
/// any specific type. Calling these methods incorrectly will result in `nng`
/// returning an error code.
pub trait OptOps: super::Opt
{
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
	/// Rawn `nng` funcion for getting an integer otion.
	const GETOPT_INT: unsafe extern "C" fn(Self::Handle, *const c_char, *mut c_int) -> c_int;
	/// Raw `nng` function to get an `nng_duration`.
	const GETOPT_MS: unsafe extern "C" fn(
		Self::Handle,
		*const c_char,
		*mut nng_sys::nng_duration,
	) -> c_int;
	/// Raw `nng` function for getting a `size_t` option.
	const GETOPT_SIZE: unsafe extern "C" fn(Self::Handle, *const c_char, *mut usize) -> c_int;
	/// Raw `nng` function for getting an `nng_sockaddr` option.
	const GETOPT_SOCKADDR: unsafe extern "C" fn(
		Self::Handle,
		*const c_char,
		*mut nng_sys::nng_sockaddr,
	) -> c_int;
	/// Raw `nng` function for getting a string value.
	const GETOPT_STRING: unsafe extern "C" fn(
		Self::Handle,
		*const c_char,
		*mut *mut c_char,
	) -> c_int;
	/// Raw `nng` function for getting a u64.
	const GETOPT_UINT64: unsafe extern "C" fn(Self::Handle, *const c_char, *mut u64) -> c_int;

	/// Raw `nng` function for setting opaque data.
	const SETOPT: unsafe extern "C" fn(Self::Handle, *const c_char, *const c_void, usize) -> c_int;
	/// Raw `nng` function for setting a boolean.
	const SETOPT_BOOL: unsafe extern "C" fn(Self::Handle, *const c_char, bool) -> c_int;
	/// Raw `nng` function to set an integer.
	const SETOPT_INT: unsafe extern "C" fn(Self::Handle, *const c_char, c_int) -> c_int;
	/// Raw `nng` function to set an `nng_duration`.
	const SETOPT_MS: unsafe extern "C" fn(
		Self::Handle,
		*const c_char,
		nng_sys::nng_duration,
	) -> c_int;
	/// Raw `nng` function to set a pointer option.
	const SETOPT_PTR: unsafe extern "C" fn(Self::Handle, *const c_char, *mut c_void) -> c_int;
	/// Raw `nng` function to set a `size_t` option.
	const SETOPT_SIZE: unsafe extern "C" fn(Self::Handle, *const c_char, usize) -> c_int;
	/// Raw `nng` function to set a string value.
	const SETOPT_STRING: unsafe extern "C" fn(Self::Handle, *const c_char, *const c_char) -> c_int;

	/// Returns the underlying `nng` type.
	fn handle(&self) -> Self::Handle;

	/// Get the boolean option.
	fn getopt_bool(&self, opt: *const c_char) -> Result<bool>
	{
		let mut raw = false;
		let rv = unsafe { (Self::GETOPT_BOOL)(self.handle(), opt, &mut raw as _) };

		rv2res!(rv, raw)
	}

	/// Get an integer option.
	fn getopt_int(&self, opt: *const c_char) -> Result<i32>
	{
		let mut res = 0;
		let rv = unsafe { (Self::GETOPT_INT)(self.handle(), opt, &mut res as _) };

		rv2res!(rv, res)
	}

	/// Get the duration from the option.
	fn getopt_ms(&self, opt: *const c_char) -> Result<Option<Duration>>
	{
		let mut dur: nng_sys::nng_duration = 0;
		let rv = unsafe { (Self::GETOPT_MS)(self.handle(), opt, &mut dur as _) };

		rv2res!(rv, crate::util::nng_to_duration(dur))
	}

	/// Get the `size_t` option.
	fn getopt_size(&self, opt: *const c_char) -> Result<usize>
	{
		let mut sz = 0;
		let rv = unsafe { (Self::GETOPT_SIZE)(self.handle(), opt, &mut sz as _) };

		rv2res!(rv, sz)
	}

	/// Get the specified socket address option.
	fn getopt_sockaddr(&self, opt: *const c_char) -> Result<SocketAddr>
	{
		unsafe {
			let mut addr: MaybeUninit<nng_sys::nng_sockaddr> = MaybeUninit::uninit();
			let rv = (Self::GETOPT_SOCKADDR)(self.handle(), opt, addr.as_mut_ptr());

			rv2res!(rv, addr.assume_init().into())
		}
	}

	/// Get the string value of the specified option.
	fn getopt_string(&self, opt: *const c_char) -> Result<String>
	{
		unsafe {
			let mut ptr: *mut c_char = ptr::null_mut();
			let rv = (Self::GETOPT_STRING)(self.handle(), opt, &mut ptr as *mut _);
			let ptr = validate_ptr(rv, ptr)?;

			let name = CStr::from_ptr(ptr.as_ptr()).to_string_lossy().into_owned();
			nng_sys::nng_strfree(ptr.as_ptr());

			Ok(name)
		}
	}

	/// The the `u64` option.
	fn getopt_uint64(&self, opt: *const c_char) -> Result<u64>
	{
		let mut res = 0;
		let rv = unsafe { (Self::GETOPT_UINT64)(self.handle(), opt, &mut res as _) };

		rv2res!(rv, res)
	}

	/// Sets the value of opaque data.
	fn setopt(&self, opt: *const c_char, val: &[u8]) -> Result<()>
	{
		let rv = unsafe { (Self::SETOPT)(self.handle(), opt, val.as_ptr() as _, val.len()) };

		rv2res!(rv)
	}

	/// Sets the value of a boolean option.
	fn setopt_bool(&self, opt: *const c_char, val: bool) -> Result<()>
	{
		let rv = unsafe { (Self::SETOPT_BOOL)(self.handle(), opt, val) };

		rv2res!(rv)
	}

	/// Set the value of an integer option.
	fn setopt_int(&self, opt: *const c_char, val: i32) -> Result<()>
	{
		let rv = unsafe { (Self::SETOPT_INT)(self.handle(), opt, val) };

		rv2res!(rv)
	}

	/// Set the duration to the option.
	fn setopt_ms(&self, opt: *const c_char, dur: Option<Duration>) -> Result<()>
	{
		let ms = crate::util::duration_to_nng(dur);

		let rv = unsafe { (Self::SETOPT_MS)(self.handle(), opt, ms) };
		rv2res!(rv)
	}

	/// Set the value of the pointer to the option.
	unsafe fn setopt_ptr(&self, opt: *const c_char, val: *mut c_void) -> Result<()>
	{
		let rv = (Self::SETOPT_PTR)(self.handle(), opt, val);
		rv2res!(rv)
	}

	/// Set the value of a `size` option.
	fn setopt_size(&self, opt: *const c_char, val: usize) -> Result<()>
	{
		let rv = unsafe { (Self::SETOPT_SIZE)(self.handle(), opt, val) };

		rv2res!(rv)
	}

	/// Set the value of the option to the value of the string.
	fn setopt_string(&self, opt: *const c_char, val: &str) -> Result<()>
	{
		let cval = CString::new(val).map_err(|_| Error::InvalidInput)?;
		let rv = unsafe { (Self::SETOPT_STRING)(self.handle(), opt, cval.as_ptr()) };

		rv2res!(rv)
	}
}
