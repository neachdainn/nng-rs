//! Options available to configure `nng` constructs.
use crate::error::Result;

/// Hidden types used by the options system
mod inner
{
	use std::os::raw::c_char;
	use crate::error::Result;

	/// Marks a type that can get and set `nng` options
	pub trait HasOpts: Sized
	{
		fn getopt_sz(&self, opt: *const c_char) -> Result<usize>;
	}

	pub trait Opt
	{
		type OptType;

		fn get<T: HasOpts>(s: &T) -> Result<Self::OptType>;
	}

	impl HasOpts for crate::socket::Socket
	{
		fn getopt_sz(&self, opt: *const c_char) -> Result<usize>
		{
			unimplemented!();
		}
	}
}

pub trait GetOpt<T: inner::Opt>: inner::HasOpts
{
	fn get_opt(&self) -> Result<T::OptType>
	{
		<T as inner::Opt>::get(self)
	}
}

/// Maximum receivable size.
pub struct MaxRecvSize;
impl inner::Opt for MaxRecvSize
{
	type OptType = usize;

	fn get<T: inner::HasOpts>(s: &T) -> Result<usize>
	{
		s.getopt_sz(nng_sys::NNG_OPT_RECVMAXSZ)
	}
}

impl GetOpt<MaxRecvSize> for crate::socket::Socket {}
