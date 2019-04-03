//! Message handling utilities
//!
//! Applications desiring to use the richest part of `nng` will want to use the
//! message API, where the message structure is passed between functions. This
//! API provides the most power support for zero-copy.
//!
//! Messages are divided into a header and a body, where the body generally
//! carries user-payload and the header carries protocol specific header
//! information. Most applications will only interact with the body.
use std::ops::{Deref, DerefMut};
use std::{ptr, slice};

use crate::error::Result;
use crate::pipe::Pipe;

/// An `nng` message type.
#[derive(Debug)]
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
	/// Creates a message from the given `nng_msg`
	pub(crate) unsafe fn from_ptr(msgp: *mut nng_sys::nng_msg) -> Self
	{
		Message { msgp, body: Body { msgp }, header: Header { msgp } }
	}

	/// Consumes the message and returns the `nng_msg` pointer.
	pub(crate) unsafe fn into_ptr(self) -> *mut nng_sys::nng_msg
	{
		let ptr = self.msgp;
		std::mem::forget(self);

		ptr
	}

	/// Create an empty message.
	pub fn new() -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_msg_alloc(&mut msgp as _, 0) };

		validate_ptr!(rv, msgp);
		Ok(unsafe { Message::from_ptr(msgp) })
	}

	/// Create an empty message with a pre-allocated body buffer.
	///
	/// The returned buffer will have a capacity equal to `cap` but a length of
	/// zero. To get a `Message` with a specified length, use `Message::zeros`.
	pub fn with_capacity(cap: usize) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_msg_alloc(&mut msgp as _, cap) };

		validate_ptr!(rv, msgp);

		// When nng allocates a message, it fills the body and sets the size to
		// whatever you requested. It makes sense in a C context, less so here.
		unsafe {
			nng_sys::nng_msg_clear(msgp);
		}

		Ok(unsafe { Message::from_ptr(msgp) })
	}

	/// Create a message that is filled to `size` with zeros.
	pub fn zeros(size: usize) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_msg_alloc(&mut msgp as _, size) };

		validate_ptr!(rv, msgp);
		Ok(unsafe { Message::from_ptr(msgp) })
	}

	/// Attempts to convert a buffer into a message.
	///
	/// This is functionally equivalent to calling `From` but allows the user
	/// to handle the case of `nng` being out of memory.
	///
	/// This function will be converted to the `TryFrom` trait once it is
	/// stable.
	pub fn try_from(s: &[u8]) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_msg_alloc(&mut msgp as _, s.len()) };

		validate_ptr!(rv, msgp);

		// At this point, `nng` thinks we have the requested amount of memory.
		// There is no more validation we can try to do.
		unsafe { ptr::copy_nonoverlapping(s.as_ptr(), nng_sys::nng_msg_body(msgp) as _, s.len()) }

		Ok(unsafe { Message::from_ptr(msgp) })
	}

	/// Attempts to duplicate the message.
	///
	/// This is functionally equivalent to calling `Clone` but allows the user
	/// to handle the case of `nng` being out of memory.
	pub fn try_clone(&self) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();

		let rv = unsafe { nng_sys::nng_msg_dup(&mut msgp as _, self.msgp) };

		validate_ptr!(rv, msgp);
		Ok(unsafe { Message::from_ptr(msgp) })
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

	/// Returns the pipe object associated with the message.
	///
	/// On receive, this is the pipe from which the message was received. On
	/// transmit, this would be the pipe that the message should be delivered
	/// to, if a specific peer is required. Note that not all protocols support
	/// overriding the destination pipe.
	///
	/// The most usual use case for this is to obtain information about the peer
	/// from which the message was received. This can be used to provide
	/// different behaviors for different peers, such as a higher level of
	/// authentication for peers located on an untrusted network.
	pub fn pipe(&mut self) -> Option<Pipe>
	{
		let (pipe, id) = unsafe {
			let pipe = nng_sys::nng_msg_get_pipe(self.msgp);
			let id = nng_sys::nng_pipe_id(pipe);
			(pipe, id)
		};

		if id > 0 {
			Some(Pipe::from_nng_sys(pipe))
		}
		else {
			None
		}
	}

	/// Sets the pipe associated with the message.
	///
	/// This is most useful when used with protocols that support directing a
	/// message to a specific peer. For example, the _pair_ version 1 protocol
	/// can do this when in polyamorous mode. Not all protocols support this.
	pub fn set_pipe(&mut self, pipe: &Pipe)
	{
		unsafe { nng_sys::nng_msg_set_pipe(self.msgp, pipe.handle()) }
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
unsafe impl Send for Message {}
unsafe impl Sync for Message {}

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

impl<'a> From<&'a [u8]> for Message
{
	fn from(s: &[u8]) -> Message
	{
		// As with `Clone`, this section is different than the rest of this
		// wrapper. Since the message allocation function only ever returns
		// `ENOMEM`, we're going to provide a more Rust-like interface by
		// panicking in the same way all other Rust allocations panic.
		Message::try_from(s).expect("Nng failed to allocate the memory")
	}
}

impl Deref for Message
{
	type Target = Body;

	fn deref(&self) -> &Body
	{
		&self.body
	}
}
impl DerefMut for Message
{
	fn deref_mut(&mut self) -> &mut Body
	{
		&mut self.body
	}
}

/// The body of a `Message`.
#[derive(Debug)]
pub struct Body
{
	msgp: *mut nng_sys::nng_msg,
}
impl Body
{
	/// Appends the data to the back of the message body.
	pub fn push_back(&mut self, data: &[u8]) -> Result<()>
	{
		let rv = unsafe { nng_sys::nng_msg_append(self.msgp, data.as_ptr() as _, data.len()) };

		rv2res!(rv)
	}

	/// Shortens the message body, keeping the first `len` bytes.
	///
	/// If `len` is greater than the message body's current length, this has no
	/// effect.
	pub fn truncate(&mut self, len: usize)
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
		let rv = unsafe { nng_sys::nng_msg_insert(self.msgp, data.as_ptr() as _, data.len()) };

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

	/// Remove the first `len` bytes from the front of the message body.
	pub fn trim(&mut self, len: usize) -> Result<()>
	{
		let rv = unsafe { nng_sys::nng_msg_trim(self.msgp, len) };

		rv2res!(rv)
	}
}
unsafe impl Send for Body {}
unsafe impl Sync for Body {}

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
#[derive(Debug)]
pub struct Header
{
	msgp: *mut nng_sys::nng_msg,
}
impl Header
{
	/// Appends the data to the back of the message header.
	pub fn push_back(&mut self, data: &[u8]) -> Result<()>
	{
		let rv =
			unsafe { nng_sys::nng_msg_header_append(self.msgp, data.as_ptr() as _, data.len()) };

		rv2res!(rv)
	}

	/// Shortens the message header, keeping the first `len` bytes.
	///
	/// If `len` is greater than the message header's current length, this has
	/// no effect.
	pub fn truncate(&mut self, len: usize)
	{
		let rv = unsafe {
			let current_len = nng_sys::nng_msg_header_len(self.msgp);
			nng_sys::nng_msg_header_chop(self.msgp, current_len.saturating_sub(len))
		};

		// We are guarding against this, so this should never happen
		assert!(rv == 0, "Message was too short to truncate");
	}

	/// Clears the message header.
	pub fn clear(&mut self)
	{
		unsafe {
			nng_sys::nng_msg_header_clear(self.msgp);
		}
	}

	/// Prepends the data to the message header.
	pub fn push_front(&mut self, data: &[u8]) -> Result<()>
	{
		let rv =
			unsafe { nng_sys::nng_msg_header_insert(self.msgp, data.as_ptr() as _, data.len()) };

		rv2res!(rv)
	}

	/// Remove the first `len` bytes from the front of the message header.
	pub fn trim(&mut self, len: usize) -> Result<()>
	{
		let rv = unsafe { nng_sys::nng_msg_header_trim(self.msgp, len) };

		rv2res!(rv)
	}
}
unsafe impl Send for Header {}
unsafe impl Sync for Header {}

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
