//! Bus (routing) example.
//!
//! The bus protocol is useful for routing applications or for building fully interconnected mesh
//! networks. In this pattern, messages are sent to every directly connected peer.
//!
//! This example was derived from [this NNG example][1].
//!
//! [1]: https://nanomsg.org/gettingstarted/nng/bus.html
use nng::{Error, Protocol, Socket};
use std::{env, process, str, thread, time::Duration};

/// Entry point of the application
fn main() -> Result<(), Error> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: bus <NODE_NAME> <URL> <URL> ...");
        process::exit(1);
    }

    node(&args[1], &args[2], &args[2..])
}

/// Bus node.
fn node(name: &str, listen: &str, dial: &[String]) -> Result<(), Error> {
    let s = Socket::new(Protocol::Bus0)?;
    s.listen(listen)?;

    // Give time for peers to bind.
    thread::sleep(Duration::from_secs(1));
    for peer in dial {
        s.dial(peer)?;
    }

    // SEND
    println!("{0}: SENDING \"{0}\" ONTO BUS", name);
    s.send(name.as_bytes())?;

    // RECV
    loop {
        let msg = s.recv()?;
        let peer = str::from_utf8(&msg).expect("invalid UTF-8");

        println!("{}: RECEIVED \"{}\" FROM BUS", name, peer);
    }
}
