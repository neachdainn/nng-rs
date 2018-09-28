#![allow(non_camel_case_types)]
///! FFI Bindings for nanomsg-next-generation
// This file defines things in the same order as "nng.h" in order to make it
// easier to spot changes between versions.

use std::os::raw::*;
type size_t = usize;

/// Macro for making constant c-strings
///
/// This macro cleans up the process of converting a `&[u8] into a `*const
/// c_char`. The caller is required to make sure the string ends in an null
/// character as I couldn't figure out a way to do that in the macro itself.
macro_rules! cstring
{
	($i:ident, $e:expr) => (
		pub const $i: *const c_char = $e as *const _ as *const c_char;
	)
}

pub mod protocol;
pub mod supplemental;

pub const NNG_MAJOR_VERSION: c_int = 1;
pub const NNG_MINOR_VERSION: c_int = 0;
pub const NNG_PATCH_VERSION: c_int = 0;
cstring!(NNG_RELEASE_SUFFIX, b"\0");

pub const NNG_MAXADDRLEN: c_int = 128;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_ctx { pub id: u32 }

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_dialer { pub id: u32 }

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_listener { pub id: u32 }

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_pipe { pub id: u32 }

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_socket { pub id: u32 }

pub type nng_duration = i32;
pub enum nng_msg {}
pub enum nng_snapshot {}
pub enum nng_stat {}
pub enum nng_aio {}

pub const NNG_PIPE_INITIALIZER: nng_pipe = nng_pipe { id: 0 };
pub const NNG_SOCKET_INITIALIZER: nng_socket = nng_socket { id: 0 };
pub const NNG_DIALER_INITIALIZER: nng_dialer = nng_dialer { id: 0 };
pub const NNG_LISTENER_INITIALIZER: nng_listener = nng_listener { id: 0 };
pub const NNG_CTX_INITIALIZER: nng_ctx = nng_ctx { id: 0 };

#[repr(C)]
#[derive(Copy, Clone)]
pub struct nng_sockaddr_inproc
{
	pub sa_family: u16,
	pub sa_name: [c_char; NNG_MAXADDRLEN as usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct nng_sockaddr_path
{
	pub sa_family: u16,
	pub sa_name: [c_char; NNG_MAXADDRLEN as usize],
}
pub type nng_sockaddr_ipc = nng_sockaddr_path;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_sockaddr_in6
{
	pub sa_family: u16,
	pub sa_port: u16,
	pub sa_addr: [u8; 16],
}
pub type nng_sockaddr_udp6 = nng_sockaddr_in6;
pub type nng_sockaddr_tcp6 = nng_sockaddr_in6;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_sockaddr_in
{
	pub sa_family: u16,
	pub sa_port: u16,
	pub sa_addr: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_sockaddr_zt
{
	pub sa_family: u16,
	pub sa_nwid: u64,
	pub sa_nodeid: u64,
	pub sa_port: u32,
}

pub type nng_sockaddr_udp = nng_sockaddr_in;
pub type nng_sockaddr_tcp = nng_sockaddr_in;

#[repr(C)]
#[derive(Copy, Clone)]
pub union nng_sockaddr
{
	pub s_family: u16,
	pub s_ipc: nng_sockaddr_ipc,
	pub s_inproc: nng_sockaddr_inproc,
	pub s_in6: nng_sockaddr_in6,
	pub s_in: nng_sockaddr_in,
	pub s_zt: nng_sockaddr_zt,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum nng_sockaddr_family
{
	NNG_AF_UNSPEC = 0,
	NNG_AF_INPROC = 1,
	NNG_AF_IPC    = 2,
	NNG_AF_INET   = 3,
	NNG_AF_INET6  = 4,
	NNG_AF_ZT     = 5,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_iov
{
	pub iov_buf: *mut c_void,
	pub iov_len: size_t,
}

pub const NNG_DURATION_INFINITE: c_int = -1;
pub const NNG_DURATION_DEFAULT: c_int = -2;
pub const NNG_DURATION_ZERO: c_int = 0;

extern "C"
{
	pub fn nng_fini();
	pub fn nng_close(s: nng_socket) -> c_int;
	pub fn nng_socket_id(s: nng_socket) -> c_int;

	pub fn nng_setopt(s: nng_socket, opt: *const c_char, val: *const c_void, valsz: size_t) -> c_int;
	pub fn nng_setopt_bool(s: nng_socket, opt: *const c_char, bval: bool) -> c_int;
	pub fn nng_setopt_int(s: nng_socket, opt: *const c_char, ival: c_int) -> c_int;
	pub fn nng_setopt_ms(s: nng_socket, opt: *const c_char, dur: nng_duration) -> c_int;
	pub fn nng_setopt_size(s: nng_socket, opt: *const c_char, z: size_t) -> c_int;
	pub fn nng_setopt_uint64(s: nng_socket, opt: *const c_char, _u64: u64) -> c_int;
	pub fn nng_setopt_string(s: nng_socket, opt: *const c_char, _str: *const c_char) -> c_int;
	pub fn nng_setopt_ptr(s: nng_socket, opt: *const c_char, ptr: *mut c_void) -> c_int;

	pub fn nng_getopt(s: nng_socket, opt: *const c_char, val: *mut c_void, valszp: *mut size_t) -> c_int;
	pub fn nng_getopt_bool(s: nng_socket, opt: *const c_char, bvalp: *mut bool) -> c_int;
	pub fn nng_getopt_int(s: nng_socket, opt: *const c_char, ivalp: *mut c_int) -> c_int;
	pub fn nng_getopt_ms(s: nng_socket, opt: *const c_char, durp: *mut nng_duration) -> c_int;
	pub fn nng_getopt_size(s: nng_socket, opt: *const c_char, zp: *mut size_t) -> c_int;
	pub fn nng_getopt_uint64(s: nng_socket, opt: *const c_char, u64p: *mut u64) -> c_int;
	pub fn nng_getopt_ptr(s: nng_socket, opt: *const c_char, ptr: *mut *mut c_void) -> c_int;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum nng_pipe_ev
{
	NNG_PIPE_EV_ADD_PRE,
	NNG_PIPE_EV_ADD_POST,
	NNG_PIPE_EV_REM_POST,
	NNG_PIPE_EV_NUM,
}

pub type nng_pipe_cb = extern "C" fn(nng_pipe, c_int, *mut c_void);

extern "C"
{
	pub fn nng_pipe_notify(s: nng_socket, ev: c_int, cb: nng_pipe_cb, arg: *mut c_void) -> c_int;
	pub fn nng_getopt_string(s: nng_socket, opt: *const c_char, strp: *mut *mut c_char) -> c_int;
	pub fn nng_listen(s: nng_socket, url: *const c_char, lp: *mut nng_listener, flags: c_int) -> c_int;
	pub fn nng_dial(s: nng_socket, url: *const c_char, dp: *mut nng_dialer, flags: c_int) -> c_int;
	pub fn nng_dialer_create(dialerp: *mut nng_dialer, s: nng_socket, url: *const c_char) -> c_int;
	pub fn nng_listener_create(listenerp: *mut nng_listener, s: nng_socket, url: *const c_char) -> c_int;
	pub fn nng_dialer_start(d: nng_dialer, flags: c_int) -> c_int;
	pub fn nng_listener_start(l: nng_listener, flags: c_int) -> c_int;
	pub fn nng_dialer_close(d: nng_dialer) -> c_int;
	pub fn nng_listener_close(l: nng_listener) -> c_int;
	pub fn nng_dialer_id(d: nng_dialer) -> c_int;
	pub fn nng_listener_id(l: nng_listener) -> c_int;

	pub fn nng_dialer_setopt(d: nng_dialer, opt: *const c_char, val: *const c_void, valsz: size_t) -> c_int;
	pub fn nng_dialer_setopt_bool(d: nng_dialer, opt: *const c_char, bval: bool) -> c_int;
	pub fn nng_dialer_setopt_int(d: nng_dialer, opt: *const c_char, ival: c_int) -> c_int;
	pub fn nng_dialer_setopt_ms(d: nng_dialer, opt: *const c_char, dur: nng_duration) -> c_int;
	pub fn nng_dialer_setopt_size(d: nng_dialer, opt: *const c_char, z: size_t) -> c_int;
	pub fn nng_dialer_setopt_uint64(d: nng_dialer, opt: *const c_char, _u64: u64) -> c_int;
	pub fn nng_dialer_setopt_string(d: nng_dialer, opt: *const c_char, _str: *const c_char) -> c_int;
	pub fn nng_dialer_setopt_ptr(d: nng_dialer, opt: *const c_char, ptr: *mut c_void) -> c_int;

	pub fn nng_dialer_getopt(d: nng_dialer, opt: *const c_char, val: *mut c_void, valszp: *mut size_t) -> c_int;
	pub fn nng_dialer_getopt_bool(d: nng_dialer, opt: *const c_char, bvalp: *mut bool) -> c_int;
	pub fn nng_dialer_getopt_int(d: nng_dialer, opt: *const c_char, ivalp: *mut c_int) -> c_int;
	pub fn nng_dialer_getopt_ms(d: nng_dialer, opt: *const c_char, durp: *mut nng_duration) -> c_int;
	pub fn nng_dialer_getopt_size(d: nng_dialer, opt: *const c_char, zp: *mut size_t) -> c_int;
	pub fn nng_dialer_getopt_sockaddr(d: nng_dialer, opt: *const c_char, sap: *mut nng_sockaddr) -> c_int;
	pub fn nng_dialer_getopt_uint64(d: nng_dialer, opt: *const c_char, u64p: *mut u64) -> c_int;
	pub fn nng_dialer_getopt_ptr(d: nng_dialer, opt: *const c_char, ptr: *mut *mut c_void) -> c_int;
	pub fn nng_dialer_getopt_string(d: nng_dialer, opt: *const c_char, strp: *mut *mut c_char) -> c_int;

	pub fn nng_listener_setopt(d: nng_listener, opt: *const c_char, val: *const c_void, valsz: size_t) -> c_int;
	pub fn nng_listener_setopt_bool(d: nng_listener, opt: *const c_char, bval: bool) -> c_int;
	pub fn nng_listener_setopt_int(d: nng_listener, opt: *const c_char, ival: c_int) -> c_int;
	pub fn nng_listener_setopt_ms(d: nng_listener, opt: *const c_char, dur: nng_duration) -> c_int;
	pub fn nng_listener_setopt_size(d: nng_listener, opt: *const c_char, z: size_t) -> c_int;
	pub fn nng_listener_setopt_uint64(d: nng_listener, opt: *const c_char, _u64: u64) -> c_int;
	pub fn nng_listener_setopt_string(d: nng_listener, opt: *const c_char, _str: *const c_char) -> c_int;
	pub fn nng_listener_setopt_ptr(d: nng_listener, opt: *const c_char, ptr: *mut c_void) -> c_int;

	pub fn nng_listener_getopt(d: nng_listener, opt: *const c_char, val: *mut c_void, valszp: *mut size_t) -> c_int;
	pub fn nng_listener_getopt_bool(d: nng_listener, opt: *const c_char, bvalp: *mut bool) -> c_int;
	pub fn nng_listener_getopt_int(d: nng_listener, opt: *const c_char, ivalp: *mut c_int) -> c_int;
	pub fn nng_listener_getopt_ms(d: nng_listener, opt: *const c_char, durp: *mut nng_duration) -> c_int;
	pub fn nng_listener_getopt_size(d: nng_listener, opt: *const c_char, zp: *mut size_t) -> c_int;
	pub fn nng_listener_getopt_sockaddr(d: nng_listener, opt: *const c_char, sap: *mut nng_sockaddr) -> c_int;
	pub fn nng_listener_getopt_uint64(d: nng_listener, opt: *const c_char, u64p: *mut u64) -> c_int;
	pub fn nng_listener_getopt_ptr(d: nng_listener, opt: *const c_char, ptr: *mut *mut c_void) -> c_int;
	pub fn nng_listener_getopt_string(d: nng_listener, opt: *const c_char, strp: *mut *mut c_char) -> c_int;

	pub fn nng_strerror(err: c_int) -> *const c_char;

	pub fn nng_send(s: nng_socket, data: *mut c_void, size: size_t, flags: c_int) -> c_int;
	pub fn nng_recv(s: nng_socket, data: *mut c_void, sizep: *mut size_t, flags: c_int) -> c_int;
	pub fn nng_sendmsg(s: nng_socket, msg: *mut nng_msg, flags: c_int) -> c_int;
	pub fn nng_recvmsg(s: nng_socket, msg: *mut *mut nng_msg, flags: c_int) -> c_int;

	pub fn nng_send_aio(s: nng_socket, aio: *mut nng_aio);
	pub fn nng_recv_aio(s: nng_socket, aio: *mut nng_aio);

	pub fn nng_ctx_open(ctxp: *mut nng_ctx, s: nng_socket) -> c_int;
	pub fn nng_ctx_close(ctx: nng_ctx) -> c_int;
	pub fn nng_ctx_id(ctx: nng_ctx) -> c_int;
	pub fn nng_ctx_recv(ctx: nng_ctx, aio: *mut nng_aio);
	pub fn nng_ctx_send(ctx: nng_ctx, aio: *mut nng_aio);
	
	pub fn nng_ctx_getopt(d: nng_ctx, opt: *const c_char, val: *mut c_void, valszp: *mut size_t) -> c_int;
	pub fn nng_ctx_getopt_bool(d: nng_ctx, opt: *const c_char, bvalp: *mut bool) -> c_int;
	pub fn nng_ctx_getopt_int(d: nng_ctx, opt: *const c_char, ivalp: *mut c_int) -> c_int;
	pub fn nng_ctx_getopt_ms(d: nng_ctx, opt: *const c_char, durp: *mut nng_duration) -> c_int;
	pub fn nng_ctx_getopt_size(d: nng_ctx, opt: *const c_char, zp: *mut size_t) -> c_int;

	pub fn nng_ctx_setopt(d: nng_ctx, opt: *const c_char, val: *const c_void, valsz: size_t) -> c_int;
	pub fn nng_ctx_setopt_bool(d: nng_ctx, opt: *const c_char, bval: bool) -> c_int;
	pub fn nng_ctx_setopt_int(d: nng_ctx, opt: *const c_char, ival: c_int) -> c_int;
	pub fn nng_ctx_setopt_ms(d: nng_ctx, opt: *const c_char, dur: nng_duration) -> c_int;
	pub fn nng_ctx_setopt_size(d: nng_ctx, opt: *const c_char, z: size_t) -> c_int;

	pub fn nng_alloc(size: size_t) -> *mut c_void;
	pub fn nng_free(ptr: *mut c_void, size: size_t);
	pub fn nng_strdup(src: *const c_char) -> *mut c_char;
	pub fn nng_strfree(str: *mut c_char);

	pub fn nng_aio_alloc(aiop: *mut *mut nng_aio, callb: extern "C" fn(*mut c_void), arg: *mut c_void) -> c_int;
	pub fn nng_aio_free(aio: *mut nng_aio);
	pub fn nng_aio_stop(aio: *mut nng_aio);
	pub fn nng_aio_result(aio: *mut nng_aio) -> c_int;
	pub fn nng_aio_count(aio: *mut nng_aio) -> size_t;
	pub fn nng_aio_cancel(aio: *mut nng_aio);
	pub fn nng_aio_abort(aio: *mut nng_aio, err: c_int);
	pub fn nng_aio_wait(aio: *mut nng_aio);
	pub fn nng_aio_set_msg(aio: *mut nng_aio, msg: *mut nng_msg);
	pub fn nng_aio_get_msg(aio: *mut nng_aio) -> *mut nng_msg;
	pub fn nng_aio_set_input(aio: *mut nng_aio, index: c_uint, param: *mut c_void);
	pub fn nng_aio_get_input(aio: *mut nng_aio, index: c_uint) -> *mut c_void;
	pub fn nng_aio_set_output(aio: *mut nng_aio, index: c_uint, result: *mut c_void);
	pub fn nng_aio_get_output(aio: *mut nng_aio, index: c_uint) -> *mut c_void;
	pub fn nng_aio_set_timeout(aio: *mut nng_aio, timeout: nng_duration);
	pub fn nng_aio_set_iov(aio: *mut nng_aio, niov: c_uint, iov: *mut nng_iov) -> c_int;
	pub fn nng_aio_finish(aio: *mut nng_aio, err: c_int);
	pub fn nng_sleep_aio(msec: nng_duration, aio: *mut nng_aio);

	pub fn nng_msg_alloc(msgp: *mut *mut nng_msg, size: size_t) -> c_int;
	pub fn nng_msg_free(msg: *mut nng_msg);
	pub fn nng_msg_realloc(msg: *mut nng_msg, size: size_t) -> c_int;
	pub fn nng_msg_header(msg: *mut nng_msg) -> *mut c_void;
	pub fn nng_msg_header_len(msg: *const nng_msg) -> size_t;
	pub fn nng_msg_body(msg: *mut nng_msg) -> *mut c_void;
	pub fn nng_msg_len(msg: *const nng_msg) -> size_t;
	pub fn nng_msg_append(msg: *mut nng_msg, val: *const c_void, size: size_t) -> c_int;
	pub fn nng_msg_insert(msg: *mut nng_msg, val: *const c_void, size: size_t) -> c_int;
	pub fn nng_msg_trim(msg: *mut nng_msg, size: size_t) -> c_int;
	pub fn nng_msg_chop(msg: *mut nng_msg, size: size_t) -> c_int;
	pub fn nng_msg_header_append(msg: *mut nng_msg, val: *const c_void, size: size_t) -> c_int;
	pub fn nng_msg_header_insert(msg: *mut nng_msg, val: *const c_void, size: size_t) -> c_int;
	pub fn nng_msg_header_trim(msg: *mut nng_msg, size: size_t) -> c_int;
	pub fn nng_msg_header_chop(msg: *mut nng_msg, size: size_t) -> c_int;
	pub fn nng_msg_header_append_u32(msg: *mut nng_msg, val32: u32) -> c_int;
	pub fn nng_msg_header_insert_u32(msg: *mut nng_msg, val32: u32) -> c_int;
	pub fn nng_msg_header_chop_u32(msg: *mut nng_msg, val32: *mut u32) -> c_int;
	pub fn nng_msg_header_trim_u32(msg: *mut nng_msg, val32: *mut u32) -> c_int;
	pub fn nng_msg_append_u32(msg: *mut nng_msg, val32: u32) -> c_int;
	pub fn nng_msg_insert_u32(msg: *mut nng_msg, val32: u32) -> c_int;
	pub fn nng_msg_chop_u32(msg: *mut nng_msg, val32: *mut u32) -> c_int;
	pub fn nng_msg_trim_u32(msg: *mut nng_msg, val32: *mut u32) -> c_int;

	pub fn nng_msg_dup(dup: *mut *mut nng_msg, orig: *const nng_msg) -> c_int;
	pub fn nng_msg_clear(msg: *mut nng_msg);
	pub fn nng_msg_header_clear(msg: *mut nng_msg);
	pub fn nng_msg_set_pipe(msg: *mut nng_msg, pipe: nng_pipe);
	pub fn nng_msg_get_pipe(msg: *const nng_msg) -> nng_pipe;
	pub fn nng_msg_getopt(msg: *mut nng_msg, opt: c_int, ptr: *mut c_void, szp: *mut size_t) -> c_int;

	pub fn nng_pipe_getopt(d: nng_pipe, opt: *const c_char, val: *mut c_void, valszp: *mut size_t) -> c_int;
	pub fn nng_pipe_getopt_bool(d: nng_pipe, opt: *const c_char, bvalp: *mut bool) -> c_int;
	pub fn nng_pipe_getopt_int(d: nng_pipe, opt: *const c_char, ivalp: *mut c_int) -> c_int;
	pub fn nng_pipe_getopt_ms(d: nng_pipe, opt: *const c_char, durp: *mut nng_duration) -> c_int;
	pub fn nng_pipe_getopt_size(d: nng_pipe, opt: *const c_char, zp: *mut size_t) -> c_int;
	pub fn nng_pipe_getopt_sockaddr(d: nng_pipe, opt: *const c_char, sap: *mut nng_sockaddr) -> c_int;
	pub fn nng_pipe_getopt_uint64(d: nng_pipe, opt: *const c_char, u64p: *mut u64) -> c_int;
	pub fn nng_pipe_getopt_ptr(d: nng_pipe, opt: *const c_char, ptr: *mut *mut c_void) -> c_int;
	pub fn nng_pipe_getopt_string(d: nng_pipe, opt: *const c_char, strp: *mut *mut c_char) -> c_int;
	pub fn nng_pipe_close(pipe: nng_pipe) -> c_int;
	pub fn nng_pipe_id(pipe: nng_pipe) -> c_int;
	pub fn nng_pipe_socket(pipe: nng_pipe) -> nng_socket;
	pub fn nng_pipe_dialer(pipe: nng_pipe) -> nng_dialer;
	pub fn nng_pipe_listener(pipe: nng_pipe) -> nng_listener;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub enum nng_flag_enum
{
	NNG_FLAG_ALLOC    = 1,
	NNG_FLAG_NONBLOCK = 2,
}

cstring!(NNG_OPT_SOCKNAME, b"socket-name\0");
cstring!(NNG_OPT_RAW, b"raw\0");
cstring!(NNG_OPT_PROTO, b"protocol\0");
cstring!(NNG_OPT_PROTONAME, b"protocol-name\0");
cstring!(NNG_OPT_PEER, b"peer\0");
cstring!(NNG_OPT_PEERNAME, b"peer-name\0");
cstring!(NNG_OPT_RECVBUF, b"recv-buffer\0");
cstring!(NNG_OPT_SENDBUF, b"send-buffer\0");
cstring!(NNG_OPT_RECVFD, b"recv-fd\0");
cstring!(NNG_OPT_SENDFD, b"send-fd\0");
cstring!(NNG_OPT_RECVTIMEO, b"recv-timeout\0");
cstring!(NNG_OPT_SENDTIMEO, b"send-timeout\0");
cstring!(NNG_OPT_LOCADDR, b"local-address\0");
cstring!(NNG_OPT_REMADDR, b"remote-address\0");
cstring!(NNG_OPT_URL, b"url\0");
cstring!(NNG_OPT_MAXTTL, b"ttl-max\0");
cstring!(NNG_OPT_RECVMAXSZ, b"recv-size-max\0");
cstring!(NNG_OPT_RECONNMINT, b"reconnect-time-min\0");
cstring!(NNG_OPT_RECONNMAXT, b"reconnect-time-max\0");

cstring!(NNG_OPT_TLS_CONFIG, b"tls-config\0");
cstring!(NNG_OPT_TLS_AUTH_MODE, b"tls-authmode\0");
cstring!(NNG_OPT_TLS_CERT_KEY_FILE, b"tls-cert-key-file\0");
cstring!(NNG_OPT_TLS_CA_FILE, b"tls-ca-file\0");
cstring!(NNG_OPT_TLS_SERVER_NAME, b"tls-server-name\0");
cstring!(NNG_OPT_TLS_VERIFIED, b"tls-verified\0");
cstring!(NNG_OPT_TCP_NODELAY, b"tcp-nodelay\0");
cstring!(NNG_OPT_TCP_KEEPALIVE, b"tcp-keepalive\0");

#[repr(C)]
#[derive(Copy, Clone)]
pub enum nng_stat_type_enum
{
	NNG_STAT_LEVEL   = 0,
	NNG_STAT_COUNTER = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum nng_unit_enum
{
	NNG_UNIT_NONE     = 0,
	NNG_UNIT_BYTES    = 1,
	NNG_UNIT_MESSAGES = 2,
	NNG_UNIT_BOOLEAN  = 3,
	NNG_UNIT_MILLIS   = 4,
	NNG_UNIT_EVENTS   = 5,
}

extern "C"
{
	pub fn nng_device(s1: nng_socket, s2: nng_socket) -> c_int;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum nng_errno_enum {
	NNG_EINTR        = 1,
	NNG_ENOMEM       = 2,
	NNG_EINVAL       = 3,
	NNG_EBUSY        = 4,
	NNG_ETIMEDOUT    = 5,
	NNG_ECONNREFUSED = 6,
	NNG_ECLOSED      = 7,
	NNG_EAGAIN       = 8,
	NNG_ENOTSUP      = 9,
	NNG_EADDRINUSE   = 10,
	NNG_ESTATE       = 11,
	NNG_ENOENT       = 12,
	NNG_EPROTO       = 13,
	NNG_EUNREACHABLE = 14,
	NNG_EADDRINVAL   = 15,
	NNG_EPERM        = 16,
	NNG_EMSGSIZE     = 17,
	NNG_ECONNABORTED = 18,
	NNG_ECONNRESET   = 19,
	NNG_ECANCELED    = 20,
	NNG_ENOFILES     = 21,
	NNG_ENOSPC       = 22,
	NNG_EEXIST       = 23,
	NNG_EREADONLY    = 24,
	NNG_EWRITEONLY   = 25,
	NNG_ECRYPTO      = 26,
	NNG_EPEERAUTH    = 27,
	NNG_ENOARG       = 28,
	NNG_EAMBIGUOUS   = 29,
	NNG_EBADTYPE     = 30,
	NNG_EINTERNAL    = 1000,
	NNG_ESYSERR      = 0x10000000,
	NNG_ETRANERR     = 0x20000000,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct nng_url
{
	pub u_rawurl: *mut c_char,
	pub u_scheme: *mut c_char,
	pub u_userinfo: *mut c_char,
	pub u_host: *mut c_char,
	pub u_hostname: *mut c_char,
	pub u_port: *mut c_char,
	pub u_path: *mut c_char,
	pub u_query: *mut c_char,
	pub u_fragment: *mut c_char,
	pub u_requir: *mut c_char,
}

extern "C"
{
	pub fn nng_url_parse(urlp: *mut *mut nng_url, str: *const c_char) -> c_int;
	pub fn nng_url_free(url: *mut nng_url);
	pub fn nng_url_clone(dup: *mut *mut nng_url, orig: *mut nng_url) -> c_int;

	pub fn nng_version() -> *const c_char;
}
