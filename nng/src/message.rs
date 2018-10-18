use std::{ptr, slice};
use std::ops::{Deref, DerefMut};

use nng_sys;

use error::{ErrorKind, Result};

/// An `nng` message type.
///
/// Messages are divided into a header and body, where the body generally
/// carries user-payload and the header carries protocol specific header
/// information. Most applications will only interact with the body.
pub struct Message
{
	// We would like to be able to return a reference to the body and the head,
	// but they aren't accessible structures. We could create a `Body` and
	// `BodyMut` types a la iterators but that leads to a whole lot of
	// duplicated code. Instead, we're going to make them members of this
	// struct and return references to that. This will solve the borrowing
	// issue and avoid code duplication.

	/// The pointer to the actual message.
	msgp: *mut nng_sys::nng_msg,

	/// The fake "body" of the message.
	body: Body,

	/// The fake "header" of the message.
	header: Header,
}
impl Message
{
	/// Create a message from the given `nng_msg`.
	///
	/// This function mostly exists to help avoid the case where one forgets to
	/// set all three of the message pointers correctly.
	fn from_ptr(msgp: *mut nng_sys::nng_msg) -> Self
	{
		Message {
			msgp,
			body: Body { msgp },
			header: Header { msgp },
		}
	}

	/// Create an empty message.
	pub fn new() -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe {
			nng_sys::nng_msg_alloc(&mut msgp as _, 0)
		};

		if rv != 0 {
			return Err(ErrorKind::from_code(rv).into())
		}
		assert!(msgp != ptr::null_mut(), "Null pointer from successful `nng` call");

		Ok(Message::from_ptr(msgp))
	}

	/// Create an empty message with a pre-allocated body buffer.
	pub fn with_capacity(cap: usize) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe {
			nng_sys::nng_msg_alloc(&mut msgp as _, cap)
		};

		if rv != 0 {
			return Err(ErrorKind::from_code(rv).into());
		}
		assert!(msgp != ptr::null_mut(), "Null pointer from successful `nng` call");

		// When nng allocates a message, it fills the body and sets the size to
		// whatever you requested. It makes sense in a C context, less so here.
		unsafe { nng_sys::nng_msg_clear(msgp); }

		Ok(Message::from_ptr(msgp))
	}

	/// Attempts to duplicate the message.
	///
	/// This is functionally equivalent to calling `Clone` but allows the user
	/// to handle the case of `nng` being out of memory.
	pub fn try_clone(&self) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();

		let rv = unsafe {
			nng_sys::nng_msg_dup(&mut msgp as _, self.msgp)
		};

		if rv != 0 {
			return Err(ErrorKind::from_code(rv).into());
		}
		assert!(msgp != ptr::null_mut(), "Null pointer from successful `nng` call");

		Ok(Message::from_ptr(msgp))
	}

	/// Returns a reference to the message body.
	pub fn body(&self) -> &Body
	{
		&self.body
	}

	/// Returns a mutable reference to the message body.
	pub fn body_mut(&mut self) -> &mut Body
	{
		&mut self.body
	}

	/// Returns a reference to the message header.
	pub fn header(&self) -> &Header
	{
		&self.header
	}

	/// Returns a mutable reference to the message header.
	pub fn header_mut(&mut self) -> &mut Header
	{
		&mut self.header
	}
}
impl Drop for Message
{
	fn drop(&mut self)
	{
		unsafe {
			nng_sys::nng_msg_free(self.msgp);
		}
	}
}

impl Clone for Message
{
	fn clone(&self) -> Self
	{
		// This is a section of code that disagrees with the rest of this
		// library. At the time of writing, I let the `ENOMEM` error propagate
		// to the caller when `nng` doesn't have enough memory. However,
		// cloning is such a well-used part of Rust that we're going to panic
		// if the clone fails.
		self.try_clone().expect("Nng failed to duplicate the message")
	}
}

/// The body of a `Message`.
pub struct Body
{
	msgp: *mut nng_sys::nng_msg,
}
impl Body
{
	/// Appends the data to the back of the message body.
	pub fn push_back(&mut self, data: &[u8]) -> Result<()>
	{
		let rv = unsafe {
			nng_sys::nng_msg_append(self.msgp, data.as_ptr() as _, data.len())
		};

		rv2res!(rv)
	}

	/// Shortens the message body, keeping the first `len` bytes.
	///
	/// If `len` is greater than the message body's current length, this has no
	/// effect.
	pub fn truncate_back(&mut self, len: usize)
	{
		let rv = unsafe {
			let current_len = nng_sys::nng_msg_len(self.msgp);
			nng_sys::nng_msg_chop(self.msgp, current_len.saturating_sub(len))
		};

		// We are guarding against this, so this should never happen
		assert!(rv == 0, "Message was too short to truncate");
	}

	/// Clears the message body.
	pub fn clear(&mut self)
	{
		unsafe {
			nng_sys::nng_msg_clear(self.msgp);
		}
	}

	/// Prepends the data to the message body.
	pub fn push_front(&mut self, data: &[u8]) -> Result<()>
	{
		let rv = unsafe {
			nng_sys::nng_msg_insert(self.msgp, data.as_ptr() as _, data.len())
		};

		rv2res!(rv)
	}

	/// Reserves capacity for at least `additional` more bytes to be inserted.
	///
	/// This function does nothing if the capacity is already sufficient.
	pub fn reserve(&mut self, additional: usize) -> Result<()>
	{
		let rv = unsafe {
			let current_len = nng_sys::nng_msg_len(self.msgp);
			nng_sys::nng_msg_realloc(self.msgp, current_len + additional)
		};

		rv2res!(rv)
	}

	/// Remove the first `len` bytes from the front of the message
	pub fn trim(&mut self, len: usize) -> Result<()>
	{
		let rv = unsafe {
			nng_sys::nng_msg_trim(self.msgp, len)
		};

		rv2res!(rv)
	}
}
impl Deref for Body
{
	type Target = [u8];

	fn deref(&self) -> &[u8]
	{
		unsafe {
			let ptr = nng_sys::nng_msg_body(self.msgp);
			let len = nng_sys::nng_msg_len(self.msgp);

			slice::from_raw_parts(ptr as _, len)
		}
	}
}
impl DerefMut for Body
{
	fn deref_mut(&mut self) -> &mut [u8]
	{
		unsafe {
			let ptr = nng_sys::nng_msg_body(self.msgp);
			let len = nng_sys::nng_msg_len(self.msgp);

			slice::from_raw_parts_mut(ptr as _, len)
		}
	}
}

/// The header of a `Message`.
pub struct Header
{
	msgp: *mut nng_sys::nng_msg,
}
impl Deref for Header
{
	type Target = [u8];

	fn deref(&self) -> &[u8]
	{
		unsafe {
			let ptr = nng_sys::nng_msg_header(self.msgp);
			let len = nng_sys::nng_msg_header_len(self.msgp);

			slice::from_raw_parts(ptr as _, len)
		}
	}
}
impl DerefMut for Header
{
	fn deref_mut(&mut self) -> &mut [u8]
	{
		unsafe {
			let ptr = nng_sys::nng_msg_header(self.msgp);
			let len = nng_sys::nng_msg_header_len(self.msgp);

			slice::from_raw_parts_mut(ptr as _, len)
		}
	}
}
