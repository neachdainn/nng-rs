# Rust Wrapper for nanomsg-next-generation (nng)

This crate provides a safe wrapper around the [nng][1] library.
Most features are complete and the library is usable in its current state.

The `nng` library is compiled and linked by default.
If this is not the desired functionality (i.e., linking to the system installed `nng` is preferred), it can be disabled by setting `default-features` to `false`.

[1]: https://nanomsg.github.io/nng/
