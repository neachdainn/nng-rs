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
/// ```
/// use nng::options::{Options, Raw};
/// use nng::*;
///
/// let socket = Socket::new(Protocol::Pub0).unwrap();
/// let raw = socket.get_opt::<Raw>().unwrap();
/// assert!(!raw);
/// ```
///
/// But all this is a compile error:
///
/// ```compile_fail
/// use nng::options::{Options, Raw};
/// use nng::*;
///
/// let socket = Socket::new(Protocol::Pub0).unwrap();
/// socket.set_opt::<Raw>(true).unwrap(); // Won't compile
/// ```
pub trait Options: private::HasOpts
{
	/// Reads the specified option from the object.
	#[allow(clippy::missing_errors_doc)]
	fn get_opt<T: private::OptOps>(&self) -> Result<T::OptType>
	where
		Self: GetOpt<T>,
	{
		T::get(self)
	}

	/// Writes the specified option to the object.
	#[allow(clippy::missing_errors_doc)]
	fn set_opt<T: private::OptOps>(&self, val: T::OptType) -> Result<()>
	where
		Self: SetOpt<T>,
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
pub trait GetOpt<T: private::OptOps>: private::HasOpts
{
}

/// Marks that a type can set the specific `nng` option.
pub trait SetOpt<T: private::OptOps>: private::HasOpts
{
}
