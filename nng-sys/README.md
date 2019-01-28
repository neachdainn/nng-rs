# Bindings for nanomsg-next-generation (nng)

[![docs.rs](https://docs.rs/nng-sys/badge.svg)](https://docs.rs/nng-sys)
[![crates.io](http://img.shields.io/crates/v/nng-sys.svg)](http://crates.io/crates/nng-sys)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rustc 1.31+](https://img.shields.io/badge/rustc-1.31+-lightgray.svg)
![Pipeline](https://gitlab.com/neachdainn/nng-rs/badges/master/pipeline.svg)

This crate provides the FFI bindings for [nng][1].
In the future, the major and minor components of the version will be guaranteed to match the major and minor versions of the corresponding `nng` version.
However, it is currently in active development along with [the nng crate][2] and so the versions do not match.

The `nng` library is compiled and linked by default.
This can be disabled by setting `default-features` to `false`.

**Currently Linked Version:** v1.1.1

[1]: https://nanomsg.github.io/nng/
[2]: https://crates.io/crates/nng
