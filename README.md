# A safe Rust wrapper for NNG

[![docs.rs](https://docs.rs/nng/badge.svg)](https://docs.rs/nng)
[![crates.io](http://img.shields.io/crates/v/nng.svg)](http://crates.io/crates/nng)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rustc 1.31+](https://img.shields.io/badge/rustc-1.31+-lightgray.svg)
![Pipeline](https://gitlab.com/neachdainn/nng-rs/badges/master/pipeline.svg)

## What Is NNG

From the [NNG Github Repository][1]:

> NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less library, offering a simple API to solve common recurring messaging problems, such as publish/subscribe, RPC-style request/reply, or service discovery.
> The API frees the programmer from worrying about details like connection management, retries, and other common considerations, so that they can focus on the application instead of the plumbing.

## Nng-rs

This crate provides a safe wrapper around the NNG library, seeking to maintain an API that is similar to the original library.
As such, the majority of examples available online should be easy to apply to this crate.

### Rust Version Requirements

The current version requires **Rustc v1.36 or greater**.
In general, this crate should always be able to compile with the Rustc version available on the oldest Ubuntu LTS release.
Any change that requires a newer Rustc version will always be considered a breaking change and this crate's version number will be bumped accordingly.

### Features

* `build-nng` (default): Build NNG from source and statically link to the library.
* `ffi-module`: Expose the raw FFI bindings via the `nng::ffi` module.
  This is useful for utilizing NNG features that are implemented in the base library but not this wrapper.
  Note that this exposes some internal items of this library and it directly exposes the NNG library, so anything enabled by this can change without bumping versions.

### Building NNG

**NOTE:** Due to a [bug](https://github.com/nanomsg/nng/commit/3890f856068542c6ddb7b498a2e313b026450bd2) in NNG v1.2.3, building NNG currently requires enabling the statistics feature.

Enabling the `build-nng` feature will cause the NNG library to be built using the default settings and CMake generator.
Most of the time, this should just work.
However, in the case that the default are not the desired settings, there are three ways to change the build:

1. [Patch][5] the `nng-sys` dependency and enable the desired build features.
2. Disable the `build-nng` feature and directly depend on `nng-sys`.
3. Disable the `build-nng` feature and manually compile NNG.

The build features are not exposed in this crate because Cargo features are currently [strictly additive][6] and there is no way to specify mutually exclusive features (i.e., build settings).
Additionally, it does not seem very ergonomic to have this crate expose all of the same build features as the binding crate, which could lead to feature pollution in any dependent crates.

Merge requests for a better solution to this are more than welcome.

### Examples

The following example uses the [intra-process][2] transport to set up a [request][3]/[reply][4]
socket pair. The "client" sends a string to the "server" which responds with a nice phrase.

```rust
use nng::*;

const ADDRESS: &'static str = "inproc://nng/example";

fn request() -> Result<()> {
    // Set up the client and connect to the specified address
    let client = Socket::new(Protocol::Req0)?;
    client.dial(ADDRESS)?;

    // Send the request from the client to the server. In general, it will be
    // better to directly use a `Message` to enable zero-copy, but that doesn't
    // matter here.
    client.send("Ferris".as_bytes())?;

    // Wait for the response from the server.
    let msg = client.recv()?;
    assert_eq!(&msg[..], b"Hello, Ferris");
    Ok(())
}

fn reply() -> Result<()> {
    // Set up the server and listen for connections on the specified address.
    let server = Socket::new(Protocol::Rep0)?;
    server.listen(ADDRESS)?;

    // Receive the message from the client.
    let mut msg = server.recv()?;
    assert_eq!(&msg[..], b"Ferris");

    // Reuse the message to be more efficient.
    msg.push_front(b"Hello, ");

    server.send(msg)?;
    Ok(())
}
```

Additional examples are in the `examples` directory.

[1]: https://github.com/nanomsg/nng
[2]: https://nanomsg.github.io/nng/man/v1.1.0/nng_inproc.7
[3]: https://nanomsg.github.io/nng/man/v1.1.0/nng_req.7
[4]: https://nanomsg.github.io/nng/man/v1.1.0/nng_rep.7
[5]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-patch-section
[6]: https://github.com/rust-lang/cargo/issues/2980
