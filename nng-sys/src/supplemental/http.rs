use std::os::raw::*;

use crate::{nng_aio, nng_url};
use crate::supplemental::tls::nng_tls_config;

type size_t = usize;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum nng_http_status
{
	NNG_HTTP_STATUS_CONTINUE                 = 100,
	NNG_HTTP_STATUS_SWITCHING                = 101,
	NNG_HTTP_STATUS_PROCESSING               = 102,
	NNG_HTTP_STATUS_OK                       = 200,
	NNG_HTTP_STATUS_CREATED                  = 201,
	NNG_HTTP_STATUS_ACCEPTED                 = 202,
	NNG_HTTP_STATUS_NOT_AUTHORITATIVE        = 203,
	NNG_HTTP_STATUS_NO_CONTENT               = 204,
	NNG_HTTP_STATUS_RESET_CONTENT            = 205,
	NNG_HTTP_STATUS_PARTIAL_CONTENT          = 206,
	NNG_HTTP_STATUS_MULTI_STATUS             = 207,
	NNG_HTTP_STATUS_ALREADY_REPORTED         = 208,
	NNG_HTTP_STATUS_IM_USED                  = 226,
	NNG_HTTP_STATUS_MULTIPLE_CHOICES         = 300,
	NNG_HTTP_STATUS_STATUS_MOVED_PERMANENTLY = 301,
	NNG_HTTP_STATUS_FOUND                    = 302,
	NNG_HTTP_STATUS_SEE_OTHER                = 303,
	NNG_HTTP_STATUS_NOT_MODIFIED             = 304,
	NNG_HTTP_STATUS_USE_PROXY                = 305,
	NNG_HTTP_STATUS_TEMPORARY_REDIRECT       = 307,
	NNG_HTTP_STATUS_PERMANENT_REDIRECT       = 308,
	NNG_HTTP_STATUS_BAD_REQUEST              = 400,
	NNG_HTTP_STATUS_UNAUTHORIZED             = 401,
	NNG_HTTP_STATUS_PAYMENT_REQUIRED         = 402,
	NNG_HTTP_STATUS_FORBIDDEN                = 403,
	NNG_HTTP_STATUS_NOT_FOUND                = 404,
	NNG_HTTP_STATUS_METHOD_NOT_ALLOWED       = 405,
	NNG_HTTP_STATUS_NOT_ACCEPTABLE           = 406,
	NNG_HTTP_STATUS_PROXY_AUTH_REQUIRED      = 407,
	NNG_HTTP_STATUS_REQUEST_TIMEOUT          = 408,
	NNG_HTTP_STATUS_CONFLICT                 = 409,
	NNG_HTTP_STATUS_GONE                     = 410,
	NNG_HTTP_STATUS_LENGTH_REQUIRED          = 411,
	NNG_HTTP_STATUS_PRECONDITION_FAILED      = 412,
	NNG_HTTP_STATUS_PAYLOAD_TOO_LARGE        = 413,
	NNG_HTTP_STATUS_ENTITY_TOO_LONG          = 414,
	NNG_HTTP_STATUS_UNSUPPORTED_MEDIA_TYPE   = 415,
	NNG_HTTP_STATUS_RANGE_NOT_SATISFIABLE    = 416,
	NNG_HTTP_STATUS_EXPECTATION_FAILED       = 417,
	NNG_HTTP_STATUS_TEAPOT                   = 418,
	NNG_HTTP_STATUS_UNPROCESSABLE_ENTITY     = 422,
	NNG_HTTP_STATUS_LOCKED                   = 423,
	NNG_HTTP_STATUS_FAILED_DEPENDENCY        = 424,
	NNG_HTTP_STATUS_UPGRADE_REQUIRED         = 426,
	NNG_HTTP_STATUS_PRECONDITION_REQUIRED    = 428,
	NNG_HTTP_STATUS_TOO_MANY_REQUESTS        = 429,
	NNG_HTTP_STATUS_HEADERS_TOO_LARGE        = 431,
	NNG_HTTP_STATUS_UNAVAIL_LEGAL_REASONS    = 451,
	NNG_HTTP_STATUS_INTERNAL_SERVER_ERROR    = 500,
	NNG_HTTP_STATUS_NOT_IMPLEMENTED          = 501,
	NNG_HTTP_STATUS_BAD_GATEWAY              = 502,
	NNG_HTTP_STATUS_SERVICE_UNAVAILABLE      = 503,
	NNG_HTTP_STATUS_GATEWAY_TIMEOUT          = 504,
	NNG_HTTP_STATUS_HTTP_VERSION_NOT_SUPP    = 505,
	NNG_HTTP_STATUS_VARIANT_ALSO_NEGOTIATES  = 506,
	NNG_HTTP_STATUS_INSUFFICIENT_STORAGE     = 507,
	NNG_HTTP_STATUS_LOOP_DETECTED            = 508,
	NNG_HTTP_STATUS_NOT_EXTENDED             = 510,
	NNG_HTTP_STATUS_NETWORK_AUTH_REQUIRED    = 511,
}

pub enum nng_http_req {}
extern "C"
{
	pub fn nng_http_req_alloc(reqp: *mut *mut nng_http_req, url: *const nng_url) -> c_int;
	pub fn nng_http_req_free(req: *mut nng_http_req);
	pub fn nng_http_req_get_method(req: *mut nng_http_req) -> *const c_char;
	pub fn nng_http_req_get_version(req: *mut nng_http_req) -> *const c_char;
	pub fn nng_http_req_get_uri(req: *mut nng_http_req) -> *const c_char;
	pub fn nng_http_req_set_header(req: *mut nng_http_req, key: *const c_char, val: *const c_char) -> c_int;
	pub fn nng_http_req_add_header(req: *mut nng_http_req, key: *const c_char, val: *const c_char) -> c_int;
	pub fn nng_http_req_del_header(req: *mut nng_http_req, key: *const c_char) -> c_int;
	pub fn nng_http_req_get_header(req: *mut nng_http_req, key: *const c_char) -> *const c_char;
	pub fn nng_http_req_set_method(req: *mut nng_http_req, method: *const c_char) -> c_int;
	pub fn nng_http_req_set_version(req: *mut nng_http_req, version: *const c_char) -> c_int;
	pub fn nng_http_req_set_uri(req: *mut nng_http_req, uri: *const c_char) -> c_int;
	pub fn nng_http_req_set_data(req: *mut nng_http_req, body: *const c_void, size: size_t) -> c_int;
	pub fn nng_http_req_copy_data(req: *mut nng_http_req, body: *const c_void, size: size_t) -> c_int;
	pub fn nng_http_req_get_data(req: *mut nng_http_req, body: *mut *mut c_void, size: *mut size_t);
}

pub enum nng_http_res {}
extern "C"
{
	pub fn nng_http_res_alloc(resp: *mut *mut nng_http_res) -> c_int;
	pub fn nng_http_res_alloc_error(resp: *mut *mut nng_http_res, error: u16) -> c_int;
	pub fn nng_http_res_free(res: *mut nng_http_res);
	pub fn nng_http_res_get_status(res: *mut nng_http_res) -> u16;
	pub fn nng_http_res_set_status(res: *mut nng_http_res, status: u16) -> c_int;
	pub fn nng_http_res_get_reason(res: *mut nng_http_res) -> *const c_char;
	pub fn nng_http_res_set_reason(res: *mut nng_http_res, reason: *const c_char) -> c_int;
	pub fn nng_http_res_set_header(res: *mut nng_http_res, key: *const c_char, val: *const c_char) -> c_int;
	pub fn nng_http_res_add_header(res: *mut nng_http_res, key: *const c_char, val: *const c_char) -> c_int;
	pub fn nng_http_res_del_header(res: *mut nng_http_res, key: *const c_char) -> c_int;
	pub fn nng_http_res_get_header(res: *mut nng_http_res, key: *const c_char) -> *const c_char;
	pub fn nng_http_res_set_version(res: *mut nng_http_res, version: *const c_char) -> c_int;
	pub fn nng_http_res_get_version(res: *mut nng_http_res) -> *const c_char;
	pub fn nng_http_res_get_data(res: *mut nng_http_res, body: *mut *mut c_void, size: *mut size_t);
	pub fn nng_http_res_set_data(res: *mut nng_http_res, body: *const c_void, size: size_t) -> c_int;
	pub fn nng_http_res_copy_data(res: *mut nng_http_res, body: *const c_void, size: size_t) -> c_int;
}

pub enum nng_http_conn {}
extern "C"
{
	pub fn nng_http_conn_close(conn: *mut nng_http_conn);
	pub fn nng_http_conn_read(conn: *mut nng_http_conn, aio: *mut nng_aio);
	pub fn nng_http_conn_read_all(conn: *mut nng_http_conn, aio: *mut nng_aio);
	pub fn nng_http_conn_write(conn: *mut nng_http_conn, aio: *mut nng_aio);
	pub fn nng_http_conn_write_all(conn: *mut nng_http_conn, aio: *mut nng_aio);
	pub fn nng_http_conn_write_req(conn: *mut nng_http_conn, req: *mut nng_http_req, aio: *mut nng_aio);
	pub fn nng_http_conn_write_res(conn: *mut nng_http_conn, res: *mut nng_http_res, aio: *mut nng_aio);
	pub fn nng_http_conn_read_req(conn: *mut nng_http_conn, req: *mut nng_http_req, aio: *mut nng_aio);
	pub fn nng_http_conn_read_res(conn: *mut nng_http_conn, res: *mut nng_http_res, aio: *mut nng_aio);

	pub fn nng_http_req_reset(req: *mut nng_http_req);
	pub fn nng_http_res_reset(res: *mut nng_http_res);
}

pub enum nng_http_handler {}
extern "C"
{
	pub fn nng_http_handler_alloc(hp: *mut *mut nng_http_handler, path: *const c_char, func: extern "C" fn(*mut nng_aio)) -> c_int;
	pub fn nng_http_handler_free(h: *mut nng_http_handler);
	pub fn nng_http_handler_alloc_file(hp: *mut *mut nng_http_handler, path: *const c_char, filename: *const c_char) -> c_int;
	pub fn nng_http_handler_alloc_static(hp: *mut *mut nng_http_handler, path: *const c_char, data: *const c_void, size: size_t, content_type: *const c_char) -> c_int;
	pub fn nng_http_handler_alloc_redirect(hp: *mut *mut nng_http_handler, uri: *const c_char, status: u16, _where: *const c_char) -> c_int;
	pub fn nng_http_handler_alloc_directory(hp: *mut *mut nng_http_handler, path: *const c_char, dirname: *const c_char) -> c_int;
	pub fn nng_http_handler_set_method(h: *mut nng_http_handler, method: *const c_char) -> c_int;
	pub fn nng_http_handler_set_host(h: *mut nng_http_handler, host: *const c_char) -> c_int;
	pub fn nng_http_handler_collect_body(h: *mut nng_http_handler, want: bool, len: size_t) -> c_int;
	pub fn nng_http_handler_set_tree(h: *mut nng_http_handler) -> c_int;
	pub fn nng_http_handler_set_data(h: *mut nng_http_handler, data: *mut c_void, dtor: extern "C" fn(*mut c_void)) -> c_int;
	pub fn nng_http_handler_get_data(h: *mut nng_http_handler) -> *mut c_void;
}

pub enum nng_http_server {}
extern "C"
{
	pub fn nng_http_server_hold(serverp: *mut *mut nng_http_server, url: *const nng_url) -> c_int;
	pub fn nng_http_server_release(server: *mut nng_http_server) -> c_int;
	pub fn nng_http_server_start(server: *mut nng_http_server) -> c_int;
	pub fn nng_http_server_stop(server: *mut nng_http_server);
	pub fn nng_http_server_add_handler(server: *mut nng_http_server, h: *mut nng_http_handler) -> c_int;
	pub fn nng_http_server_del_handler(server: *mut nng_http_server, h: *mut nng_http_handler) -> c_int;
	pub fn nng_http_server_set_tls(server: *mut nng_http_server, cfg: *mut nng_tls_config) -> c_int;
	pub fn nng_http_server_get_tls(server: *mut nng_http_server, cfgp: *mut *mut nng_tls_config) -> c_int;
	pub fn nng_http_server_set_error_page(server: *mut nng_http_server, code: u16, body: *const c_char) -> c_int;
	pub fn nng_http_server_set_error_file(server: *mut nng_http_server, code: u16, path: *const c_char) -> c_int;
	pub fn nng_http_server_res_error(server: *mut nng_http_server, res: *mut nng_http_res) -> c_int;
	pub fn nng_http_hijack(conn: *mut nng_http_conn) -> c_int;
}

pub enum nng_http_client {}
extern "C"
{
	pub fn nng_http_client_alloc(clientp: *mut *mut nng_http_client, url: *const nng_url) -> c_int;
	pub fn nng_http_client_free(client: *mut nng_http_client);
	pub fn nng_http_client_set_tls(client: *mut nng_http_client, cfg: *mut nng_tls_config) -> c_int;
	pub fn nng_http_client_get_tls(client: *mut nng_http_client, cfgp: *mut *mut nng_tls_config) -> c_int;
	pub fn nng_http_client_connect(client: *mut nng_http_client, aio: *mut nng_aio);
	pub fn nng_http_conn_transact(conn: *mut nng_http_conn, req: *mut nng_http_req, res: *mut nng_http_res, aio: *mut nng_aio);
	pub fn nng_http_client_transact(client: *mut nng_http_client, req: *mut nng_http_req, res: *mut nng_http_res, aio: *mut nng_aio);
}
