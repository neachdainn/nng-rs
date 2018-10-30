//! Options available to configure `nng` constructs.
use crate::error::Result;

mod types;
pub use self::types::*;

pub(crate) mod private;

/// Trait for getting and setting options.
///
/// This trait allows for the getting and setting of options as long as that
/// option is available. An example of this would be the `Raw` option - it is a
/// read-only option that is available exclusively to sockets (FIXME: Not sure
/// this is the case with option fallbacks?). So the following code will work:
///
/// ```ignore
/// let raw = socket.get_opt::<Raw>()?;
/// ```
///
/// But all of these are compile errors:
///
/// ```ignore
/// let raw = dialer.get_opt::<Raw>()?;
/// socket.set_opt::<Raw>(true)?;
/// ```
pub trait Options: private::HasOpts
{
	/// Reads the specified option from the object.
	fn get_opt<T: private::Opt>(&self) -> Result<T::OptType>
		where Self: GetOpt<T>
	{
		T::get(self)
	}

	/// Writes the specified option to the object.
	fn set_opt<T: private::Opt>(&self, val: T::OptType) -> Result<()>
		where Self: SetOpt<T>
	{
		T::set(self, val)
	}
}
impl<T: private::HasOpts> Options for T {}

/// Marks that a type can get the specific `nng` option.
pub trait GetOpt<T: private::Opt>: private::HasOpts {}

/// Marks that a type can set the specific `nng` option.
pub trait SetOpt<T: private::Opt>: private::HasOpts {}
