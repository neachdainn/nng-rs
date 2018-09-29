use std::os::raw::*;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_optspec
{
	pub o_name: *const c_char,
	pub o_short: c_int,
	pub o_val: c_int,
	pub o_arg: bool,
}

extern "C"
{
	pub fn nng_opts_parse(argc: c_int, argv: *const *const c_char, opts: *const nng_optspec, val: *mut c_int, optarg: *const *const c_char, optidx: *mut c_int) -> c_int;
}
