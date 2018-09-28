use std::os::raw::*;
use nng_socket;

// The non-versioned functions depend on include order. The closest Rust
// approximation is defining them in the versioned module.

pub mod bus0
{
	use super::*;

	extern "C"
	{
		pub fn nng_bus0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_bus0_open_raw(s: *mut nng_socket) -> c_int;
	}

	pub use self::nng_bus0_open as nng_bus_open;
	pub use self::nng_bus0_open_raw as nng_bus_open_raw;
}

pub mod pair0
{
	use super::*;

	extern "C"
	{
		pub fn nng_pair0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_pair0_open_raw(s: *mut nng_socket) -> c_int;
	}

	pub use self::nng_pair0_open as nng_pair_open;
	pub use self::nng_pair0_open_raw as nng_pair_open_raw;
}

pub mod pair1
{
	use super::*;

	extern "C"
	{
		pub fn nng_pair1_open(s: *mut nng_socket) -> c_int;
		pub fn nng_pair1_open_raw(s: *mut nng_socket) -> c_int;
	}

	pub use self::nng_pair1_open as nng_pair_open;
	pub use self::nng_pair1_open_raw as nng_pair_open_raw;

	cstring!(NNG_OPT_PAIR1_POLY, b"pair1:polyamorous\0");
}

pub mod pipeline0
{
	use super::*;

	extern "C"
	{
		pub fn nng_pull0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_pull0_open_raw(s: *mut nng_socket) -> c_int;

		pub fn nng_push0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_push0_open_raw(s: *mut nng_socket) -> c_int;
	}

	pub use self::nng_pull0_open as nng_pull_open;
	pub use self::nng_pull0_open_raw as nng_pull_open_raw;

	pub use self::nng_push0_open as nng_push_open;
	pub use self::nng_push0_open_raw as nng_push_open_raw;
}

pub mod pubsub0
{
	use super::*;

	extern "C"
	{
		pub fn nng_pub0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_pub0_open_raw(s: *mut nng_socket) -> c_int;

		pub fn nng_sub0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_sub0_open_raw(s: *mut nng_socket) -> c_int;
	}

	pub use self::nng_pub0_open as nng_pub_open;
	pub use self::nng_pub0_open_raw as nng_pub_open_raw;

	pub use self::nng_sub0_open as nng_sub_open;
	pub use self::nng_sub0_open_raw as nng_sub_open_raw;

	cstring!(NNG_OPT_SUB_SUBSCRIBE, b"sub:subscribe\0");
	cstring!(NNG_OPT_SUB_UNSUBSCRIBE, b"sub:unsubscribe\0");
}

pub mod reqrep0
{
	use super::*;

	extern "C"
	{
		pub fn nng_req0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_req0_open_raw(s: *mut nng_socket) -> c_int;

		pub fn nng_rep0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_rep0_open_raw(s: *mut nng_socket) -> c_int;
	}

	pub use self::nng_req0_open as nng_req_open;
	pub use self::nng_req0_open_raw as nng_req_open_raw;

	pub use self::nng_rep0_open as nng_rep_open;
	pub use self::nng_rep0_open_raw as nng_rep_open_raw;

	cstring!(NNG_OPT_REQ_RESENDTIME, b"req:resend-time\0");
}

pub mod survey0
{
	use super::*;

	extern "C"
	{
		pub fn nng_respondent0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_respondent0_open_raw(s: *mut nng_socket) -> c_int;

		pub fn nng_surveyor0_open(s: *mut nng_socket) -> c_int;
		pub fn nng_surveyor0_open_raw(s: *mut nng_socket) -> c_int;
	}

	pub use self::nng_respondent0_open as nng_respondent_open;
	pub use self::nng_respondent0_open_raw as nng_respondent_open_raw;

	pub use self::nng_surveyor0_open as nng_surveyor_open;
	pub use self::nng_surveyor0_open_raw as nng_surveyor_open_raw;

	cstring!(NNG_OPT_SURVEYOR_SURVEYTIME, b"surveyor:survey-time\0");
}
