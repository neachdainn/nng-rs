//! The collection of macros used by this project.

/// Converts a `nng` return code into a Rust `Result`.
macro_rules! rv2res
{
	($rv:expr, $ok:expr) => (
		match $rv {
			0 => Ok($ok),
			e => Err(crate::error::ErrorKind::from_code(e).into()),
		}
	);

	($rv:expr) => ( rv2res!($rv, ()) )
}

/// Checks an `nng` return code and validates the pointer.
macro_rules! validate_ptr
{
	($rv:ident, $ptr:ident) => (
		if $rv != 0 {
			return Err(crate::error::ErrorKind::from_code($rv).into());
		}
		assert!(!$ptr.is_null(), "Nng returned a null pointer from a successful function");
	)
}

/// Utility macro for creating a new option type.
///
/// This is 90% me just playing around with macros. It is probably a terrible
/// way to go around doing this task but, then again, this whole options
/// business has been a complete mess.
macro_rules! create_option
{
	(
		$(#[$attr:meta])*
		$opt:ident -> $ot:ty:
		Get $g:ident = $gexpr:stmt;
		Set $s:ident $v:ident = $sexpr:stmt;
	) => {
		$(#[$attr])*
		pub enum $opt {}
		impl $crate::options::private::Opt for $opt
		{
			type OptType = $ot;

			fn get<T: $crate::options::private::HasOpts>($g: &T) -> $crate::error::Result<Self::OptType> { $gexpr }
			fn set<T: $crate::options::private::HasOpts>($s: &T, $v: Self::OptType) -> $crate::error::Result<()> { $sexpr }
		}
	}
}

/// Implements the specified options for the type.
macro_rules! expose_options
{
	(
		$struct:ident :: $member:ident -> $handle:ty;
		GETOPT_BOOL = $go_b:expr;
		GETOPT_INT = $go_i:expr;
		GETOPT_MS = $go_ms:expr;
		GETOPT_SIZE = $go_sz:expr;
		GETOPT_SOCKADDR = $go_sa:expr;
		GETOPT_STRING = $go_str:expr;

		SETOPT = $so:expr;
		SETOPT_BOOL = $so_b:expr;
		SETOPT_INT = $so_i:expr;
		SETOPT_MS = $so_ms:expr;
		SETOPT_SIZE = $so_sz:expr;
		SETOPT_STRING = $so_str:expr;

		Gets -> [$($($getters:ident)::+),*];
		Sets -> [$($($setters:ident)::+),*];
	) => {
		impl $crate::options::private::HasOpts for $struct
		{
			type Handle = $handle;
			fn handle(&self) -> Self::Handle { self.$member }

			const GETOPT_BOOL: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut bool) -> std::os::raw::c_int = $go_b;
			const GETOPT_INT: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut std::os::raw::c_int) -> std::os::raw::c_int = $go_i;
			const GETOPT_MS: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut nng_sys::nng_duration) -> std::os::raw::c_int = $go_ms;
			const GETOPT_SIZE: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut usize) -> std::os::raw::c_int = $go_sz;
			const GETOPT_SOCKADDR: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut nng_sys::nng_sockaddr) -> std::os::raw::c_int = $go_sa;
			const GETOPT_STRING: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *mut *mut std::os::raw::c_char) -> std::os::raw::c_int = $go_str;

			const SETOPT: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *const std::os::raw::c_void, usize) -> std::os::raw::c_int = $so;
			const SETOPT_BOOL: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, bool) -> std::os::raw::c_int = $so_b;
			const SETOPT_INT: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, std::os::raw::c_int) -> std::os::raw::c_int = $so_i;
			const SETOPT_MS: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, nng_sys::nng_duration) -> std::os::raw::c_int = $so_ms;
			const SETOPT_SIZE: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, usize) -> std::os::raw::c_int = $so_sz;
			const SETOPT_STRING: unsafe extern "C" fn(Self::Handle, *const std::os::raw::c_char, *const std::os::raw::c_char) -> std::os::raw::c_int = $so_str;
		}

		$(impl $crate::options::GetOpt<$crate::options::$($getters)::+> for $struct {})*
		$(impl $crate::options::SetOpt<$crate::options::$($setters)::+> for $struct {})*
	}
}
