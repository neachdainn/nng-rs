# A safe Rust wrapper for NNG

[![docs.rs](https://docs.rs/nng/badge.svg)](https://docs.rs/nng)
[![crates.io](http://img.shields.io/crates/v/nng.svg)](http://crates.io/crates/nng)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rustc 1.31+](https://img.shields.io/badge/rustc-1.31+-lightgray.svg)
![Pipeline](https://gitlab.com/neachdainn/nng-rs/badges/master/pipeline.svg)

## What Is NNG

From the [NNG Github Repository][1]:

> NNG, like its predecessors nanomsg (and to some extent ZeroMQ), is a lightweight, broker-less
library, offering a simple API to solve common recurring messaging problems, such as
publish/subscribe, RPC-style request/reply, or service discovery. The API frees the programmer
from worrying about details like connection management, retries, and other common
considerations, so that they can focus on the application instead of the plumbing.

## Nng-rs

This crate provides a safe wrapper around the NNG library, seeking to maintain an API that is
similar to the original library. As such, the majority of examples available online should be
easy to apply to this crate.

### Rust Version Requirements

The current version requires **Rustc v1.31 or greater**.
In general, this crate should always be able to compile with the Rustc version available on the oldest Ubuntu LTS release.
Any change that requires a newer Rustc version will always be considered a breaking change and this crate's version number will be bumped accordingly.

### Features

* `build-nng` (default): Build NNG from source and statically link to the library.
* `ffi-module`: Expose the raw FFI bindings via the `nng::ffi` module.
  This is useful for utilizing NNG features that are implemented in the base library but not this wrapper.

### Examples

The following example uses the [intra-process][2] transport to set up a [request][3]/[reply][4]
socket pair. The "client" sends a String to the "server" which responds with a nice phrase.

```rust
use std::io::Write;
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
    let reply = String::from_utf8_lossy(&msg);
    assert_eq!(reply, "Hello, Ferris!");
    Ok(())
}

fn reply() -> Result<()> {
    // Set up the server and listen for connections on the specified address.
    let server = Socket::new(Protocol::Rep0)?;
    server.listen(ADDRESS)?;

    // Receive the message from the client.
    let mut msg = server.recv()?;
    let name = String::from_utf8_lossy(&msg).into_owned();
    assert_eq!(name, "Ferris");

    // Reuse the message to be more efficient.
    msg.clear();
    write!(msg, "Hello, {}!", name).unwrap();

    server.send(msg)?;
    Ok(())
}
```

Additional examples are in the `examples` directory.

[1]: https://github.com/nanomsg/nng
[2]: https://nanomsg.github.io/nng/man/v1.1.0/nng_inproc.7
[3]: https://nanomsg.github.io/nng/man/v1.1.0/nng_req.7
[4]: https://nanomsg.github.io/nng/man/v1.1.0/nng_rep.7
