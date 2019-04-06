//! A simple asynchronous RPC server example.
//!
//! This example shows how to write an asynchronous RPC server using Contexts.
//! This is heavily based on the `nng` demonstration program of the same name.
//!
//! The protocol is simple: the client sends a request with the number of
//! milliseconds to wait, the server waits that long and sends back an empty
//! reply.
extern crate byteorder;
extern crate nng;

use std::time::{Duration, Instant};
use std::{env, mem, process, thread};

use byteorder::{ByteOrder, LittleEndian};
use nng::{Context, Message, Protocol, Socket};
use nng::aio::{AioResult, CallbackAio};

/// Number of outstanding requests that we can handle at a given time.
///
/// This is *NOT* the number of threads in use, but instead represents
/// outstanding work items. Select a small number to reduce memory size. (Each
/// one of these can be thought of as a request-reply loop.) Note that you will
/// probably run into limitations on the number of open file descriptors if you
/// set this too high. (If not for that limit, this could be set in the
/// thousands, each context consumes a couple of KB.)
const PARALLEL: usize = 128;

/// Entry point of the application.
fn main() -> Result<(), nng::Error>
{
	// Begin by parsing the arguments. We are either a server or a client, and
	// we need an address and potentially a sleep duration.
	let args: Vec<_> = env::args().collect();

	match &args[..] {
		[_, t, url] if t == "server" => server(url),
		[_, t, url, count] if t == "client" => client(url, count.parse().unwrap()),
		_ => {
			println!("Usage:\nasync server <url>\n  or\nasync client <url> <ms>");
			process::exit(1);
		},
	}
}

/// Run the client portion of the program.
fn client(url: &str, ms: u64) -> Result<(), nng::Error>
{
	let mut s = Socket::new(Protocol::Req0)?;
	s.dial(url)?;

	let mut req = Message::zeros(mem::size_of::<u64>())?;
	LittleEndian::write_u64(&mut req, ms);

	let start = Instant::now();
	s.send(req)?;
	s.recv()?;

	let dur = Instant::now().duration_since(start);
	let subsecs: u64 = dur.subsec_millis().into();
	println!("Request took {} milliseconds", dur.as_secs() * 1000 + subsecs);

	Ok(())
}

/// Run the server portion of the program.
fn server(url: &str) -> Result<(), nng::Error>
{
	// Create the socket
	let mut s = Socket::new(Protocol::Rep0)?;

	// Create all of the worker contexts
	let mut workers: Vec<_> = (0..PARALLEL)
		.map(|_| {
			let ctx = Context::new(&s)?;
			let ctx_clone = ctx.clone();
			let aio = CallbackAio::new(move |aio, res| worker_callback(aio, &ctx_clone, res))?;
			Ok((aio, ctx))
		})
		.collect::<Result<_, nng::Error>>()?;

	// Only after we have the workers do we start listening.
	s.listen(url)?;

	// Now start all of the workers listening.
	for (a, c) in &mut workers {
		c.recv(a)?;
	}

	thread::sleep(Duration::from_secs(60 * 60 * 24 * 365));

	Ok(())
}

/// Callback function for workers.
fn worker_callback(aio: &mut CallbackAio, ctx: &Context, res: AioResult)
{
	match res {
		// We successfully did nothing.
		AioResult::InactiveOk => {},

		// We successfully sent the message, wait for a new one.
		AioResult::SendOk => ctx.recv(aio).unwrap(),

		// We successfully received a message.
		AioResult::RecvOk(m) => {
			let ms = LittleEndian::read_u64(&m);
			aio.sleep(Duration::from_millis(ms)).unwrap();
		},

		// We successfully slept.
		AioResult::SleepOk => {
			let msg = Message::new().unwrap();
			ctx.send(aio, msg).unwrap();
		},

		// Anything else is an error and we will just panic.
		AioResult::SendErr(_, e) | AioResult::RecvErr(e) | AioResult::SleepErr(e) =>
			panic!("Error: {}", e),
	}
}
