use std::fmt;
use std::net::{SocketAddrV4, SocketAddrV6};
use std::os::raw::c_char;
use std::path::PathBuf;

/// Represents the addresses used by the underlying transports.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SocketAddr
{
	/// An address associated with intra-process communication.
	InProc(String),

	/// Represents an address associated with IPC communication.
	Ipc(PathBuf),

	/// Address for TCP/IP (v4) communication.
	Inet(SocketAddrV4),

	/// Address for TCP/IP (v6) communication.
	Inet6(SocketAddrV6),

	#[doc(hidden)]
	/// Used to represent a ZeroTier address.
	ZeroTier(SocketAddrZt),

	/// An invalid address type.
	#[doc(hidden)]
	Unspecified,
}

impl fmt::Display for SocketAddr
{
	/// Format trait for an empty format, `{}`.
	///
	/// Note that this is liable to change and does not necessarily map to the URL originally
	/// provided to NNG.
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		match self {
			SocketAddr::InProc(s) => write!(f, "inproc://{}", s),
			SocketAddr::Ipc(s) => write!(f, "ipc://{}", s.to_string_lossy()),
			SocketAddr::Inet(s) => write!(f, "tcp://{}", s),
			SocketAddr::Inet6(s) => write!(f, "tcp://{}", s),
			SocketAddr::ZeroTier(s) => write!(f, "zt://{}", s),
			SocketAddr::Unspecified => write!(f, "unspecified"),
		}
	}
}

impl From<nng_sys::nng_sockaddr> for SocketAddr
{
	fn from(addr: nng_sys::nng_sockaddr) -> SocketAddr
	{
		unsafe {
			match addr.s_family {
				nng_sys::NNG_AF_INPROC => {
					SocketAddr::InProc(buf_to_string(&addr.s_inproc.sa_name[..]))
				},
				nng_sys::NNG_AF_IPC => {
					SocketAddr::Ipc(buf_to_string(&addr.s_ipc.sa_path[..]).into())
				},
				nng_sys::NNG_AF_INET => {
					SocketAddr::Inet(SocketAddrV4::new(addr.s_in.sa_addr.into(), addr.s_in.sa_port))
				},
				nng_sys::NNG_AF_INET6 => SocketAddr::Inet6(SocketAddrV6::new(
					addr.s_in6.sa_addr.into(),
					addr.s_in6.sa_port,
					0,
					0,
				)),
				nng_sys::NNG_AF_ZT => SocketAddr::ZeroTier(SocketAddrZt::new(&addr.s_zt)),
				_ => SocketAddr::Unspecified,
			}
		}
	}
}

/// A ZeroTier socket address.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct SocketAddrZt
{
	pub family: u16,
	pub nwid:   u64,
	pub nodeid: u64,
	pub port:   u32,
}
impl SocketAddrZt
{
	/// Converts an `nng_sockaddr_zt` into its corresponding Rust type.
	fn new(addr: &nng_sys::nng_sockaddr_zt) -> SocketAddrZt
	{
		SocketAddrZt {
			family: addr.sa_family,
			nwid:   addr.sa_nwid,
			nodeid: addr.sa_nodeid,
			port:   addr.sa_port,
		}
	}
}
impl fmt::Display for SocketAddrZt
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
	{
		// I have no idea if this output is meaningful at all. This is just vaguely based off the
		// URI format for ZeroTier, ignoring fields that don't appear in the specification and
		// guessing how all of the others align.
		write!(f, "{}.{}:{}", self.nodeid, self.nwid, self.port)
	}
}

/// Creates a `String` from a slice that _probably_ contains UTF-8 and
/// _probably_ is null terminated.
///
/// The function is unsafe because it reinterprets the `i8` buffer as a `u8`
/// buffer via a call to `slice::from_raw_parts`.
unsafe fn buf_to_string(buf: &[c_char]) -> String
{
	// Unfortunately, the Rust standard library doesn't have a `from_ptr_len`
	// style function that would allow me to pass in the whole buffer. Instead,
	// we need to determine if there is a null byte and only pass in the slice
	// up to that point.
	//
	// Another layer of unfortunate is that there is no owned version of
	// `String::from_utf8_lossy`, so we can either allocate twice or we can do
	// a little playing with fire. As this function is already getting called
	// from unsafe code, I don't think it is a major issue to also make this
	// unsafe.
	use std::slice;

	let len = buf.len();
	let buf = slice::from_raw_parts(&buf[0] as *const c_char as _, len);
	let null_byte = buf.iter().position(|&b| b == 0).unwrap_or(len);
	String::from_utf8_lossy(&buf[..null_byte]).into_owned()
}
