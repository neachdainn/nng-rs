use std::os::raw::*;
use nng_duration;

pub type nng_time = u64;
extern "C"
{
	pub fn nng_clock() -> nng_time;
	pub fn nng_msleep(msec: nng_duration);
}

pub enum nng_thread {}
extern "C"
{
	pub fn nng_thread_create(thrp: *mut *mut nng_thread, func: extern "C" fn(*mut c_void), arg: *mut c_void) -> c_int;
	pub fn nng_thread_destroy(thr: *mut nng_thread);
}

pub enum nng_mtx {}
extern "C"
{
	pub fn nng_mtx_alloc(mtxp: *mut *mut nng_mtx) -> c_int;
	pub fn nng_mtx_free(mtx: *mut nng_mtx);
	pub fn nng_mtx_lock(mtx: *mut nng_mtx);
	pub fn nng_mtx_unlock(mtx: *mut nng_mtx);
}

pub enum nng_cv {}
extern "C"
{
	pub fn nng_cv_alloc(cvp: *mut *mut nng_cv, mtx: *mut nng_mtx) -> c_int;
	pub fn nng_cv_free(cv: *mut nng_cv);
	pub fn nng_cv_wait(cv: *mut nng_cv);
	pub fn nng_cv_until(cv: *mut nng_cv, when: nng_time) -> c_int;
	pub fn nng_cv_wake(cv:  *mut nng_cv);
	pub fn nng_cv_wake1(cv: *mut nng_cv);

	pub fn nng_random() -> u32;
}
