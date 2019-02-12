//! Options available to configure `nng` constructs.
//!
//! Many of the options are transport or protocol specific. Additionally, even
//! though the Socket does not have a specific transport, it is able to accept
//! transport options to be used as defaults for any new Dialers or Listeners.
//!
//! Additionally, a Dialer or Listener is able to read options from the
//! underlying Socket but they are unable to write options unless they are
//! directly supported.
use crate::error::Result;

mod types;
pub use self::types::*;

pub(crate) mod private;

/// Trait for getting and setting options.
///
/// This trait allows for the getting and setting of options as long as that
/// option is available. An example of this would be the `Raw` option - it is a
/// read-only option that is available exclusively to sockets. So the following
/// code will work:
///
/// ```ignore
/// let raw = socket.get_opt::<Raw>()?;
/// ```
///
/// But all this is a compile error:
///
/// ```ignore
/// socket.set_opt::<Raw>(true)?;
/// ```
pub trait Options: private::HasOpts
{
	/// Reads the specified option from the object.
	fn get_opt<T: private::OptOps>(&self) -> Result<T::OptType>
	where
		Self: GetOpt<T>,
	{
		T::get(self)
	}

	/// Writes the specified option to the object.
	fn set_opt<T: private::OptOps>(&self, val: T::OptType) -> Result<()>
	where
		Self: SetOpt<T>,
	{
		T::set(self, val)
	}

	/// Reads the specified option from the object.
	///
	/// The user is required to understand and enforce all invariants associated with this option.
	/// Usually, this means checking that a returned pointer is not `null`.
	///
	/// All options exposed this way should be considered unstable, as the plan is to eventually
	/// create a safe interface for them.
	unsafe fn unsafe_get_opt<T: private::OptOps>(&self) -> Result<T::OptType>
	where
		Self: UnsafeGetOpt<T>
	{
		T::get(self)
	}

	/// Writes the specified option to the object.
	///
	/// The user is required to understand and enforce all invariants associated with this option.
	/// This usually means that the provided pointer lives as long as the object receiving the
	/// option. It is particularly important to note that many NNG options do _not_ copy the value
	/// behind pointer options.
	///
	/// All options exposed this way should be considered unstable, as the plan is to eventually
	/// create a safe interface for them.
	unsafe fn unsafe_set_opt<T: private::OptOps>(&self, val: T::OptType) -> Result<()>
	where
		Self: UnsafeGetOpt<T>
	{
		T::set(self, val)
	}
}
impl<T: private::HasOpts> Options for T {}

/// Marks the type as an `nng` option.
pub trait Opt
{
	/// The type that the option read and writes.
	type OptType;
}

/// Marks that a type can get the specific `nng` option.
pub trait GetOpt<T: private::OptOps>: private::HasOpts {}

/// Marks that a type can set the specific `nng` option.
pub trait SetOpt<T: private::OptOps>: private::HasOpts {}

/// Marks that a type can (unsafely) get the specific `nng` option.
pub trait UnsafeGetOpt<T: private::OptOps>: private::HasOpts {}

/// Marks that a type can (unsafely) set the specific `nng` option.
pub trait UnsafeSetOpt<T: private::OptOps>: private::HasOpts {}
