# Rust Wrapper for nanomsg-next-generation (nng)

This crate provides a safe wrapper around the [nng][1] library.
It is currently in active development and is missing some functionality, specifically asynchronous contexts.
However, the basic socket types are ready for use.

The `nng` library is compiled and linked by default.
If this is not the desired functionality (i.e., linking to the system installed `nng` is preferred), it can be disabled by setting `default-features` to `false`.

[1]: https://nanomsg.github.io/nng/
