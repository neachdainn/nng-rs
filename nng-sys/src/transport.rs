use std::os::raw::*;

pub mod inproc
{
	use super::*;
	extern "C" { pub fn nng_inproc_register() -> c_int; }
}

pub mod ipc
{
	use super::*;
	extern "C" { pub fn nng_ipc_register() -> c_int; }

	cstring!(NNG_OPT_IPC_SECURITY_DESCRIPTOR, b"ipc:security-descriptor\0");
	cstring!(NNG_OPT_IPC_PERMISSIONS, b"ipc:permissions\0");
	cstring!(NNG_OPT_IPC_PEER_UID, b"ipc:peer-uid\0");
	cstring!(NNG_OPT_IPC_PEER_GID, b"ipc:peer-gid\0");
	cstring!(NNG_OPT_IPC_PEER_PID, b"ipc:peer-pid\0");
	cstring!(NNG_OPT_IPC_PEER_ZONEID, b"ipc:peer-zoneid\0");
}

pub mod tcp
{
	use super::*;
	extern "C" { pub fn nng_tcp_register() -> c_int; }
}

pub mod tls
{
	use super::*;
	extern "C" { pub fn nng_tls_register() -> c_int; }
}

pub mod websocket
{
	use super::*;
	extern "C"
	{
		pub fn nng_ws_register() -> c_int;
		pub fn nng_wss_register() -> c_int;
	}

	cstring!(NNG_OPT_WS_REQUEST_HEADERS, b"ws:request-headers\0");
	cstring!(NNG_OPT_WS_RESPONSE_HEADERS, b"ws:response-headers\0");
	pub use self::NNG_OPT_WS_REQUEST_HEADERS as NNG_OPT_WSS_REQUEST_HEADERS;
	pub use self::NNG_OPT_WS_RESPONSE_HEADERS as NNG_OPT_WSS_RESPONSE_HEADERS;
}

pub mod zerotier
{
	use super::*;

	cstring!(NNG_OPT_ZT_HOME, b"zt:home\0");
	cstring!(NNG_OPT_ZT_NWID, b"zt:nwid\0");
	cstring!(NNG_OPT_ZT_NODE, b"zt:node\0");
	cstring!(NNG_OPT_ZT_NETWORK_STATUS, b"zt:network-status\0");
	cstring!(NNG_OPT_ZT_NETWORK_NAME, b"zt:network-name\0");
	cstring!(NNG_OPT_ZT_PING_TIME, b"zt:ping-time\0");
	cstring!(NNG_OPT_ZT_PING_TRIES, b"zt:ping-tries\0");
	cstring!(NNG_OPT_ZT_CONN_TIME, b"zt:conn-time\0");
	cstring!(NNG_OPT_ZT_CONN_TRIES, b"zt:conn-tries\0");
	cstring!(NNG_OPT_ZT_MTU, b"zt:mtu\0");
	cstring!(NNG_OPT_ZT_ORBIT, b"zt:orbit\0");
	cstring!(NNG_OPT_ZT_DEORBIT, b"zt:deorbit\0");

	#[repr(C)]
	#[derive(Copy, Clone, Debug)]
	pub enum nng_zt_status
	{
		NNG_ZT_STATUS_UP,
		NNG_ZT_STATUS_CONFIG,
		NNG_ZT_STATUS_DENIED,
		NNG_ZT_STATUS_NOTFOUND,
		NNG_ZT_STATUS_ERROR,
		NNG_ZT_STATUS_OBSOLETE,
		NNG_ZT_STATUS_UNKNOWN,
	}

	extern "C" { pub fn nng_zt_register() -> c_int; }
}
