//! Message handling utilities
use std::{
	io::{self, Write},
	iter::FromIterator,
	ops::{Deref, DerefMut, Index, IndexMut},
	ptr::{self, NonNull},
	slice::{self, SliceIndex},
};

use crate::{error::Result, pipe::Pipe, util::validate_ptr};

/// An `nng` message type.
///
/// Applications desiring to use the richest part of `nng` will want to use the
/// message API, where the message structure is passed between functions. This
/// API provides the most powerful support for zero-copy.
///
/// In addition to the regular portion of the message there is a header that
/// carries protocol specific header information. Most applications will not
/// need to touch the header and will only interact with the regular message.
// TODO(#29): We could implement many other common traits, we just have to figure out if the header
// should be included in those or not. Maybe sometimes people will care about that. Also, make sure
// those changes also get applied to `Header`.
#[derive(Debug)]
pub struct Message
{
	/// The pointer to the actual message.
	msgp: NonNull<nng_sys::nng_msg>,

	/// The fake "header" of the message.
	///
	/// This object is fake because the Rust level message type does not
	/// actually need to have a header type, as NNG manages that behind the
	/// single pointer. However, if the user has a reference to the header than
	/// the message needs to be considered borrowed. We could make the
	/// header type similar to iterators (in that they would hold a reference to
	/// the message) but that requires both a `Header` and `HeaderMut` type
	/// which seems like it would just end with duplicate code.
	header: Header,
}
impl Message
{
	/// Create an empty message.
	pub fn new() -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_msg_alloc(&mut msgp as _, 0) };

		let msgp = validate_ptr(rv, msgp)?;
		Ok(Message::from_ptr(msgp))
	}

	/// Create an empty message with a pre-allocated body buffer.
	///
	/// The returned buffer will have a capacity equal to `cap` but a length of
	/// zero. To get a `Message` with a specified length, use `Message::zeros`.
	pub fn with_capacity(cap: usize) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_msg_alloc(&mut msgp as _, cap) };
		let msgp = validate_ptr(rv, msgp)?;

		// When nng allocates a message, it fills the body and sets the size to
		// whatever you requested. It makes sense in a C context, less so here.
		unsafe {
			nng_sys::nng_msg_clear(msgp.as_ptr());
		}

		Ok(Message::from_ptr(msgp))
	}

	/// Create a message that is filled to `size` with zeros.
	pub fn with_zeros(size: usize) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_msg_alloc(&mut msgp as _, size) };

		let msgp = validate_ptr(rv, msgp)?;
		Ok(Message::from_ptr(msgp))
	}

	/// Attempts to convert a buffer into a message.
	///
	/// This is functionally equivalent to calling `From` but allows the user
	/// to handle the case of `nng` being out of memory.
	pub fn from_slice(s: &[u8]) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();
		let rv = unsafe { nng_sys::nng_msg_alloc(&mut msgp as _, s.len()) };

		let msgp = validate_ptr(rv, msgp)?;

		// At this point, `nng` thinks we have the requested amount of memory.
		// There is no more validation we can try to do.
		unsafe {
			ptr::copy_nonoverlapping(s.as_ptr(), nng_sys::nng_msg_body(msgp.as_ptr()) as _, s.len())
		}

		Ok(Message::from_ptr(msgp))
	}

	/// Shortens the message, dropping excess elements from the back.
	///
	/// If `len` is greater than the message body's current length, this has no
	/// effect.
	pub fn truncate(&mut self, len: usize)
	{
		let rv = unsafe {
			let current_len = nng_sys::nng_msg_len(self.msgp.as_ptr());
			nng_sys::nng_msg_chop(self.msgp.as_ptr(), current_len.saturating_sub(len))
		};

		// We are guarding against this, so this should never happen
		debug_assert_eq!(rv, 0, "Message was too short to truncate");
	}

	/// Remove the first `len` bytes from the front of the message body.
	///
	/// If `len` is greater than the message body's current length then this
	/// will clear the entire message.
	pub fn trim(&mut self, len: usize)
	{
		let rv = unsafe {
			let current_len = nng_sys::nng_msg_len(self.msgp.as_ptr());
			nng_sys::nng_msg_trim(self.msgp.as_ptr(), len.min(current_len))
		};

		debug_assert_eq!(rv, 0, "Message was too short to trim");
	}

	/// Returns a slice that contains the contents of the message body.
	pub fn as_slice(&self) -> &[u8]
	{
		unsafe {
			let ptr = nng_sys::nng_msg_body(self.msgp.as_ptr());
			let len = nng_sys::nng_msg_len(self.msgp.as_ptr());

			slice::from_raw_parts(ptr as _, len)
		}
	}

	/// Returns a mutable slice that contains the contents of the message body.
	pub fn as_mut_slice(&mut self) -> &mut [u8]
	{
		unsafe {
			let ptr = nng_sys::nng_msg_body(self.msgp.as_ptr());
			let len = nng_sys::nng_msg_len(self.msgp.as_ptr());

			slice::from_raw_parts_mut(ptr as _, len)
		}
	}

	/// Returns a reference to the message header.
	pub const fn as_header(&self) -> &Header { &self.header }

	/// Returns a mutable reference to the message header.
	pub fn as_mut_header(&mut self) -> &mut Header { &mut self.header }

	/// Returns the length of the message.
	pub fn len(&self) -> usize { unsafe { nng_sys::nng_msg_len(self.msgp.as_ptr()) } }

	/// Returns true if the message body is empty.
	pub fn is_empty(&self) -> bool { self.len() == 0 }

	/// Clears the message body.
	pub fn clear(&mut self)
	{
		unsafe {
			nng_sys::nng_msg_clear(self.msgp.as_ptr());
		}
	}

	/// Prepends the data to the message body.
	pub fn push_front(&mut self, data: &[u8]) -> Result<()>
	{
		let rv =
			unsafe { nng_sys::nng_msg_insert(self.msgp.as_ptr(), data.as_ptr() as _, data.len()) };

		rv2res!(rv)
	}

	/// Appends the data to the back of the message body.
	pub fn push_back(&mut self, data: &[u8]) -> Result<()>
	{
		let rv =
			unsafe { nng_sys::nng_msg_append(self.msgp.as_ptr(), data.as_ptr() as _, data.len()) };

		rv2res!(rv)
	}

	/// Attempts to duplicate the message.
	///
	/// This is functionally equivalent to calling `Clone` but allows the user
	/// to handle the case of `nng` being out of memory.
	pub fn try_clone(&self) -> Result<Self>
	{
		let mut msgp: *mut nng_sys::nng_msg = ptr::null_mut();

		let rv = unsafe { nng_sys::nng_msg_dup(&mut msgp as _, self.msgp.as_ptr()) };

		let msgp = validate_ptr(rv, msgp)?;
		Ok(Message::from_ptr(msgp))
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
			let pipe = nng_sys::nng_msg_get_pipe(self.msgp.as_ptr());
			let id = nng_sys::nng_pipe_id(pipe);
			(pipe, id)
		};

		if id > 0 { Some(Pipe::from_nng_sys(pipe)) } else { None }
	}

	/// Sets the pipe associated with the message.
	///
	/// This is most useful when used with protocols that support directing a
	/// message to a specific peer. For example, the _pair_ version 1 protocol
	/// can do this when in polyamorous mode. Not all protocols support this.
	pub fn set_pipe(&mut self, pipe: Pipe)
	{
		unsafe { nng_sys::nng_msg_set_pipe(self.msgp.as_ptr(), pipe.handle()) }
	}

	/// Creates a new message from the given pointer.
	pub(crate) const fn from_ptr(msgp: NonNull<nng_sys::nng_msg>) -> Self
	{
		Message { msgp, header: Header { msgp } }
	}

	/// Consumes the message and returns the `nng_msg` pointer.
	pub(crate) fn into_ptr(self) -> NonNull<nng_sys::nng_msg>
	{
		let ptr = self.msgp;
		std::mem::forget(self);

		ptr
	}
}

#[cfg(feature = "ffi-module")]
impl Message
{
	/// Returns the underlying `nng_msg` pointer.
	pub fn nng_msg(&self) -> *mut nng_sys::nng_msg
	{
		self.msgp.as_ptr()
	}
}

impl Drop for Message
{
	fn drop(&mut self)
	{
		unsafe {
			nng_sys::nng_msg_free(self.msgp.as_ptr());
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

impl Default for Message
{
	fn default() -> Message { Message::new().unwrap() }
}

impl<'a> From<&'a [u8]> for Message
{
	fn from(s: &[u8]) -> Message
	{
		// As with `Clone`, this section is different than the rest of this
		// wrapper. Since the message allocation function only ever returns
		// `ENOMEM`, we're going to provide a more Rust-like interface by
		// panicking in the same way all other Rust allocations panic.
		Message::from_slice(s).expect("Nng failed to allocate the memory")
	}
}

impl<'a> From<&'a Vec<u8>> for Message
{
	fn from(s: &Vec<u8>) -> Message { s.as_slice().into() }
}

impl FromIterator<u8> for Message
{
	fn from_iter<T>(iter: T) -> Message
	where
		T: IntoIterator<Item = u8>,
	{
		let iter = iter.into_iter();
		let (lower, _) = iter.size_hint();
		let mut msg = Message::with_capacity(lower).expect("Failed to allocate memory");
		msg.extend(iter);
		msg
	}
}

impl<'a> FromIterator<&'a u8> for Message
{
	fn from_iter<T>(iter: T) -> Message
	where
		T: IntoIterator<Item = &'a u8>,
	{
		let iter = iter.into_iter();
		let (lower, _) = iter.size_hint();
		let mut msg = Message::with_capacity(lower).expect("Failed to allocate memory");
		msg.extend(iter);
		msg
	}
}

impl Deref for Message
{
	type Target = [u8];

	fn deref(&self) -> &[u8] { self.as_slice() }
}
impl DerefMut for Message
{
	fn deref_mut(&mut self) -> &mut [u8] { self.as_mut_slice() }
}

impl Write for Message
{
	#[inline]
	fn write(&mut self, buf: &[u8]) -> io::Result<usize>
	{
		self.push_back(buf)?;
		Ok(buf.len())
	}

	#[inline]
	fn write_all(&mut self, buf: &[u8]) -> io::Result<()>
	{
		self.push_back(buf)?;
		Ok(())
	}

	#[inline]
	fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl Extend<u8> for Message
{
	fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I)
	{
		for byte in iter {
			self.push_back(slice::from_ref(&byte)).expect("Failed to push to Message");
		}
	}
}

impl<'a> Extend<&'a u8> for Message
{
	fn extend<I: IntoIterator<Item = &'a u8>>(&mut self, iter: I)
	{
		for byte in iter {
			self.push_back(slice::from_ref(byte)).expect("Failed to push to Message");
		}
	}
}

impl<I: SliceIndex<[u8]>> Index<I> for Message
{
	type Output = I::Output;

	#[inline]
	fn index(&self, index: I) -> &Self::Output { self.as_slice().index(index) }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for Message
{
	#[inline]
	fn index_mut(&mut self, index: I) -> &mut Self::Output { self.as_mut_slice().index_mut(index) }
}

/// The header of a `Message`.
///
/// Most normal applications will never have to touch the message header. The
/// only time it will be necessary is if the socket is in "raw" mode.
#[derive(Debug)]
pub struct Header
{
	msgp: NonNull<nng_sys::nng_msg>,
}
impl Header
{
	/// Shortens the message header, keeping the first `len` bytes.
	///
	/// If `len` is greater than the message header's current length, this has
	/// no effect.
	pub fn truncate(&mut self, len: usize)
	{
		let rv = unsafe {
			let current_len = nng_sys::nng_msg_header_len(self.msgp.as_ptr());
			nng_sys::nng_msg_header_chop(self.msgp.as_ptr(), current_len.saturating_sub(len))
		};

		// We are guarding against this, so this should never happen
		debug_assert!(rv == 0, "Message header was too short to truncate");
	}

	/// Remove the first `len` bytes from the front of the message header.
	///
	/// If `len` is greater than the message header's current length then this
	/// will clear the entire message.
	pub fn trim(&mut self, len: usize)
	{
		let rv = unsafe {
			let current_len = nng_sys::nng_msg_header_len(self.msgp.as_ptr());
			nng_sys::nng_msg_header_trim(self.msgp.as_ptr(), len.min(current_len))
		};

		debug_assert_eq!(rv, 0, "Message header was too short to trim");
	}

	/// Returns a slice that contains the contents of the message header.
	pub fn as_slice(&self) -> &[u8]
	{
		unsafe {
			let ptr = nng_sys::nng_msg_header(self.msgp.as_ptr());
			let len = nng_sys::nng_msg_header_len(self.msgp.as_ptr());

			slice::from_raw_parts(ptr as _, len)
		}
	}

	/// Returns a mutable slice that contains the contents of the message
	/// header.
	pub fn as_mut_slice(&mut self) -> &mut [u8]
	{
		unsafe {
			let ptr = nng_sys::nng_msg_header(self.msgp.as_ptr());
			let len = nng_sys::nng_msg_header_len(self.msgp.as_ptr());

			slice::from_raw_parts_mut(ptr as _, len)
		}
	}

	/// Returns the length of the message header.
	pub fn len(&self) -> usize { unsafe { nng_sys::nng_msg_header_len(self.msgp.as_ptr()) } }

	/// Returns true if the message header is empty.
	pub fn is_empty(&self) -> bool { self.len() == 0 }

	/// Clears the message header.
	pub fn clear(&mut self)
	{
		unsafe {
			nng_sys::nng_msg_header_clear(self.msgp.as_ptr());
		}
	}

	/// Appends the data to the back of the message header.
	pub fn push_back(&mut self, data: &[u8]) -> Result<()>
	{
		let rv = unsafe {
			nng_sys::nng_msg_header_append(self.msgp.as_ptr(), data.as_ptr() as _, data.len())
		};

		rv2res!(rv)
	}

	/// Prepends the data to the message header.
	pub fn push_front(&mut self, data: &[u8]) -> Result<()>
	{
		let rv = unsafe {
			nng_sys::nng_msg_header_insert(self.msgp.as_ptr(), data.as_ptr() as _, data.len())
		};

		rv2res!(rv)
	}
}
unsafe impl Send for Header {}
unsafe impl Sync for Header {}

impl Deref for Header
{
	type Target = [u8];

	fn deref(&self) -> &[u8] { self.as_slice() }
}
impl DerefMut for Header
{
	fn deref_mut(&mut self) -> &mut [u8] { self.as_mut_slice() }
}

impl Write for Header
{
	#[inline]
	fn write(&mut self, buf: &[u8]) -> io::Result<usize>
	{
		self.push_back(buf)?;
		Ok(buf.len())
	}

	#[inline]
	fn write_all(&mut self, buf: &[u8]) -> io::Result<()>
	{
		self.push_back(buf)?;
		Ok(())
	}

	#[inline]
	fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl Extend<u8> for Header
{
	fn extend<I: IntoIterator<Item = u8>>(&mut self, iter: I)
	{
		for byte in iter {
			self.push_back(slice::from_ref(&byte)).expect("Failed to push to Message");
		}
	}
}

impl<'a> Extend<&'a u8> for Header
{
	fn extend<I: IntoIterator<Item = &'a u8>>(&mut self, iter: I)
	{
		for byte in iter {
			self.push_back(slice::from_ref(byte)).expect("Failed to push to Message");
		}
	}
}

impl<I: SliceIndex<[u8]>> Index<I> for Header
{
	type Output = I::Output;

	#[inline]
	fn index(&self, index: I) -> &Self::Output { self.as_slice().index(index) }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for Header
{
	#[inline]
	fn index_mut(&mut self, index: I) -> &mut Self::Output { self.as_mut_slice().index_mut(index) }
}
