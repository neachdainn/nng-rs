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

### Examples

The following example uses the [intra-process][2] transport to set up a [request][3]/[reply][4]
socket pair. The "client" sends a String to the "server" which responds with a nice phrase.

```rust
use nng::*;

// Set up the server and listen for connections on the specified address.
let address = "inproc://nng/lib.rs";
let server = Socket::new(Protocol::Rep0).unwrap();
server.listen(address).unwrap();

// Set up the client and connect to the specified address
let client = Socket::new(Protocol::Req0).unwrap();
client.dial(address).unwrap();

// Send the request from the client to the server.
let request = b"Ferris"[..].into();
client.send(request).unwrap();

// Receive the message on the server and send back the reply
let request = {
    let req = server.recv().unwrap();
    String::from_utf8(req.to_vec()).unwrap()
};
assert_eq!(request, "Ferris");
let reply = format!("Hello, {}!", request).as_bytes().into();
server.send(reply).unwrap();

// Get the response on the client side.
let reply = {
    let rep = client.recv().unwrap();
    String::from_utf8(rep.to_vec()).unwrap()
};
assert_eq!(reply, "Hello, Ferris!");
```

Additional examples are in the `examples` directory.

[1]: https://github.com/nanomsg/nng
[2]: https://nanomsg.github.io/nng/man/v1.1.0/nng_inproc.7
[3]: https://nanomsg.github.io/nng/man/v1.1.0/nng_req.7
[4]: https://nanomsg.github.io/nng/man/v1.1.0/nng_rep.7
