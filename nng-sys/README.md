# Bindings for nanomsg-next-generation

This crate provides the FFI bindings for [nng][1].
In the future, the major and minor components of the version will be guaranteed to match the major and minor versions of the corresponding `nng` version.
However, it is currently in active development along with [the nng crate][2] and so the versions do not match.

The `nng` library is compiled and linked by default.
This can be disabled by setting `default-features` to `false`.

[1]: https://nanomsg.github.io/nng/
[2]: https://crates.io/crates/nng
