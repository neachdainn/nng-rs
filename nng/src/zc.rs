//! Helper code for Nng's zero-copy functionality
//!
//! Using the zero-copy functionality requires allocating and freeing buffers
//! using Nng functions. This module has a lightweigh wrapper around these
//! functions to make sure that all memory is freed correctly.
use std::ops::{Deref, DerefMut};
use std::{mem, ptr, slice};

use nng_sys;

use error::{ErrorKind, Result};

/// A buffer of data that can be fed to zero-copy functions.
///
/// This structure can be used to reduce data copies, thereby increasing
/// performance, particularly if the buffer is reused to send a response.
pub struct ZeroCopyBuffer
{
	/// Pointer to the start of the buffer
	buf: *mut u8,

	/// Size of the buffer
	size: usize,
}

impl ZeroCopyBuffer
{
	/// Creates a new buffer with the specified size.
	///
	/// This function returns an `ErrorKind::OutOfMemory` error upon failure.
	pub fn with_capacity(size: usize) -> Result<Self>
	{
		let buf = unsafe { nng_sys::nng_alloc(size) } as _;

		if buf == ptr::null_mut() {
			Err(ErrorKind::OutOfMemory.into())
		} else { Ok(ZeroCopyBuffer { buf, size }) }
	}

	/// Creates a buffer from the raw parts.
	///
	/// This function assumes that the pointer is both valid and allocated by
	/// `nng` and that the size is correct.
	pub(crate) unsafe fn from_raw_parts(buf: *mut u8, size: usize) -> Self
	{
		ZeroCopyBuffer { buf, size }
	}

	/// Breaks the buffer into its raw components.
	pub(crate) fn into_raw_parts(self) -> (*mut u8, usize)
	{
		let res = (self.buf, self.size);
		mem::forget(self);

		res
	}
}

impl Deref for ZeroCopyBuffer
{
	type Target = [u8];

	fn deref(&self) -> &[u8]
	{
		unsafe {
			slice::from_raw_parts(self.buf, self.size)
		}
	}
}

impl DerefMut for ZeroCopyBuffer
{
	fn deref_mut(&mut self) -> &mut [u8]
	{
		unsafe {
			slice::from_raw_parts_mut(self.buf, self.size)
		}
	}
}

impl Drop for ZeroCopyBuffer
{
	fn drop(&mut self)
	{
		unsafe {
			nng_sys::nng_free(self.buf as _, self.size);
		}
	}
}
