use std::os::raw::*;
type size_t = usize;

pub enum nng_tls_config {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum nng_tls_mode
{
	NNG_TLS_MODE_CLIENT = 0,
	NNG_TLS_MODE_SERVER = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum nng_tls_auth_mode
{
	NNG_TLS_AUTH_MODE_NONE     = 0,
	NNG_TLS_AUTH_MODE_OPTIONAL = 1,
	NNG_TLS_AUTH_MODE_REQUIRED = 2,
}

extern "C"
{
	pub fn nng_tls_config_alloc(cfgp: *mut *mut nng_tls_config, mode: nng_tls_mode) -> c_int;
	pub fn nng_tls_config_hold(cfg: *mut nng_tls_config);
	pub fn nng_tls_config_free(cfg: *mut nng_tls_config);
	pub fn nng_tls_config_server_name(cfg: *mut nng_tls_config, name: *const c_char) -> c_int;
	pub fn nng_tls_config_ca_chain(cfg: *mut nng_tls_config, chain: *const c_char, cr1: *const c_char) -> c_int;
	pub fn nng_tls_config_own_cert(cfg: *mut nng_tls_config, cert: *const c_char, key: *const c_char, pass: *const c_char) -> c_int;
	pub fn nng_tls_config_key(cfg: *mut nng_tls_config, key: *const u8, size: size_t) -> c_int;
	pub fn nng_tls_config_pass(cfg: *mut nng_tls_config, pass: *const c_char) -> c_int;
	pub fn nng_tls_config_auth_mode(cfg: *mut nng_tls_config, mode: nng_tls_auth_mode) -> c_int;
	pub fn nng_tls_config_ca_file(cfg: *mut nng_tls_config, path: *const c_char) -> c_int;
	pub fn nng_tls_config_cert_key_file(cfg: *mut nng_tls_config, path: *const c_char, pass: *const c_char) -> c_int;
}
