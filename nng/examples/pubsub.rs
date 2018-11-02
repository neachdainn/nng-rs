//! A simple PUB/SUB demonstration application.
//!
//! This application simply publishes the seconds since the Unix epoch every
//! few seconds.
extern crate nng;
extern crate byteorder;

use std::{env, mem, process, thread};
use std::time::{Duration, SystemTime};
use nng::{Socket, Protocol, Message};
use nng::options::Options;
use nng::options::protocol::pubsub::Subscribe;
use byteorder::{ByteOrder, LittleEndian};

/// Entry point of the application.
fn main() -> Result<(), nng::Error>
{
	// Begin by parsing the arguments to determine whether this is the
	// subscriber or the publisher and what URL to connect with.
	let args: Vec<_> = env::args().take(3).collect();

	match &args[..] {
		[_, t, url] if t == "publisher"  => publisher(url),
		[_, t, url] if t == "subscriber" => subscriber(url),
		_ => {
			println!("Usage: pubsub publisher|subscriber <url>");
			process::exit(1);
		}
	}
}

/// Run the publisher portion of the program.
fn publisher(url: &str) -> Result<(), nng::Error>
{
	let mut s = Socket::new(Protocol::Pub0)?;
	s.listen(url)?;

	loop {
		// Sleep for a little bit before sending the next message.
		thread::sleep(Duration::from_secs(3));

		// Calculate the time and send it.
		let data = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("Current system time is before Unix epoch")
			.as_secs();

		let mut msg = Message::zeros(mem::size_of::<u64>())?;
		LittleEndian::write_u64(&mut msg, data);

		println!("PUBLISHER: SENDING {}", data);
		s.send(msg)?;
	}
}

/// Run the subscriber portion of the program.
fn subscriber(url: &str) -> Result<(), nng::Error>
{
	let mut s = Socket::new(Protocol::Sub0)?;
	s.dial(url)?;

	println!("SUBSCRIBER: SUBSCRIBING TO ALL TOPICS");
	let all_topics = vec![];
	s.set_opt::<Subscribe>(all_topics)?;

	loop {
		let msg = s.recv()?;
		let epoch = LittleEndian::read_u64(&msg);
		println!("SUBSCRIBER: UNIX EPOCH WAS {} SECONDS AGO", epoch);
	}
}
