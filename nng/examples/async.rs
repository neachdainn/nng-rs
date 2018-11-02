//! A simple asynchronous RPC server example.
//!
//! This example shows how to write an asynchronous RPC server using Contexts.
//! This is heavily based on the `nng` demonstration program of the same name.
//!
//! The protocol is simple: the client sends a request with the number of
//! milliseconds to wait, the server waits that long and sends back an empty
//! reply.
extern crate nng;
extern crate byteorder;

use std::{env, thread, process, mem};
use std::time::{Duration, Instant};
use nng::{Socket, Protocol, Message};
use nng::aio::{Aio, Context};
use byteorder::{ByteOrder, LittleEndian};

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
		}
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
	println!("Request took {} milliseconds", dur.as_secs() * 1000 + dur.subsec_millis() as u64);

	Ok(())
}

/// Run the server portion of the program.
fn server(url: &str) -> Result<(), nng::Error>
{
	// Create the socket
	let mut s = Socket::new(Protocol::Rep0)?;

	// Create all of the worker contexts
	let workers: Vec<_> =
		(0..PARALLEL)
		.map(|_| create_worker(&s))
		.collect::<Result<_, _>>()?;

	// Only after we have the workers do we start listening.
	s.listen(url)?;

	// Now start all of the workers listening.
	for (a, c) in &workers {
		a.recv(c)?;
	}

	thread::sleep(Duration::from_secs(60 * 60 * 24 * 365));

	Ok(())
}

/// Create a new worker context for the server.
fn create_worker(s: &Socket) -> Result<(Aio, Context), nng::Error>
{
	let mut state = State::Recv;

	let ctx = Context::new(s)?;
	let ctx_clone = ctx.clone();
	let aio = Aio::with_callback(move |aio| worker_callback(aio, &ctx_clone, &mut state))?;

	Ok((aio, ctx))
}

/// Callback function for workers.
fn worker_callback(aio: &Aio, ctx: &Context, state: &mut State)
{
	let new_state = match *state {
		State::Recv => {
			// If there was an issue, we're just going to panic instead of
			// doing something sensible.
			let _ = aio.result().unwrap();
			let msg = aio.get_msg().unwrap();
			let ms = LittleEndian::read_u64(&msg);

			aio.sleep(Duration::from_millis(ms)).unwrap();
			State::Wait
		},
		State::Wait => {
			let msg = Message::new().unwrap();
			aio.send(ctx, msg).unwrap();

			State::Send
		},
		State::Send => {
			// Again, just panic bad if things happened.
			let _ = aio.result().unwrap();
			aio.recv(ctx).unwrap();

			State::Recv
		}
	};

	*state = new_state;
}

/// State of a request.
#[derive(Copy, Clone)]
enum State { Recv, Wait, Send }
