//! Types for dealing with Nng contexts and asynchronous IO operations.
//!
//! Most applications will interact with _nng_ synchronously; that is that
//! functions such as `Scoket::send` will block the calling thread until the
//! operation has completed.
//!
//! Asynchronous operations behave differently. These operations are initiated
//! by the calling thread, but control returns immediately to the calling
//! thread. When the operation is subsequently completed (regardless of whether
//! this was successful or not), then a user supplied function ("callback") is
//! executed. An Aio object is associated with each asynchronous operation.
//!
//! Contexts allow the independent and concurrent use of stateful operations
//! using the same socket. For example, two different contexts created on a
//! _rep_ socket can each receive requests, and send replies to them, without
//! any regard to or interference with each other.
mod aio;
pub use self::aio::Aio;

mod ctx;
pub use self::ctx::Context;
