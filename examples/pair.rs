//! Pair (two way radio) example.
//!
//! The pair pattern is used when there is a one-to-one peer relationship. Only one peer may be
//! connected to another peer at a time but both may speak freely.
//!
//! This example was derived from [this NNG example][1].
//!
//! [1]: https://nanomsg.org/gettingstarted/nng/pair.html
use nng::{
    options::{Options, RecvTimeout},
    Error, Message, Protocol, Socket,
};
use std::{env, io::Write, process, str, thread, time::Duration};

/// Entry point of the application.
pub fn main() -> Result<(), Error> {
    let args: Vec<_> = env::args().take(3).collect();

    match &args[..] {
        [_, t, url] if t == "node0" => node0(url),
        [_, t, url] if t == "node1" => node1(url),
        _ => {
            println!("Usage: pipeline node0|node1 <URL> <ARG> ...");
            process::exit(1);
        }
    }
}

/// The listening node.
fn node0(url: &str) -> Result<(), Error> {
    let s = Socket::new(Protocol::Pair0)?;
    s.listen(url)?;

    send_recv(&s, "NODE0")
}

/// The dialing node.
fn node1(url: &str) -> Result<(), Error> {
    let s = Socket::new(Protocol::Pair0)?;
    s.dial(url)?;

    send_recv(&s, "NODE1")
}

/// Sends and receives messages on the socket.
fn send_recv(s: &Socket, name: &str) -> Result<(), Error> {
    s.set_opt::<RecvTimeout>(Some(Duration::from_millis(100)))?;
    loop {
        // Attempt to reuse the message if we can.
        let mut msg = match s.recv() {
            Ok(m) => {
                let partner = str::from_utf8(&m).expect("invalid UTF-8 message");
                println!("{}: RECEIVED \"{}\"", name, partner);

                m
            }

            Err(Error::TimedOut) => Message::new(),

            Err(e) => return Err(e),
        };

        thread::sleep(Duration::from_secs(1));

        msg.clear();
        write!(msg, "{}", name).expect("failed to write to message");

        println!("{0}: SENDING \"{0}\"", name);
        s.send(msg)?;
    }
}
