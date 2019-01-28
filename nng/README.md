# Rust Wrapper for nanomsg-next-generation (nng)

[![docs.rs](https://docs.rs/nng/badge.svg)](https://docs.rs/nng)
[![crates.io](http://img.shields.io/crates/v/nng.svg)](http://crates.io/crates/nng)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rustc 1.31+](https://img.shields.io/badge/rustc-1.31+-lightgray.svg)
![Pipeline](https://gitlab.com/neachdainn/nng-rs/badges/master/pipeline.svg)

This crate provides a safe wrapper around the [nng][1] library, seeking to maintain an API that is similar to the original library.
Most features are complete and the library is usable in its current state.

The `nng` library is compiled and linked by default.
If this is not the desired functionality (i.e., linking to the system installed `nng` is preferred), it can be disabled by setting `default-features` to `false`.

## Example

```rust
use nng::{Message, Protocol, Socket};

fn client() -> Result<(), nng::Error>
{
	let mut s = Socket::new(Protocol::Req0)?;
	s.dial("tcp://127.0.0.1")?;

	// Send the request to the response server
	let mut req = Message::from(&[0xDE, 0xAD, 0xBE, 0xEF])?;
	s.send(req)?;

	// Wait for the response
	let res = s.recv()?;

	println!("Response: {:?}", res);

	Ok(())
}

fn server() -> Result<(), nng::Error>
{
	let mut s = Socket::new(Protocol::Rep0)?;
	s.listen("tcp://127.0.0.1")?;

	loop {
		// Wait for a request from the client
		let mut msg = s.recv()?;


		// Respond to the client
		msg[1] += 1;
		s.send(msg)?;
	}
}
```

Additional examples are in the `examples` directory.

[1]: https://nanomsg.github.io/nng/
