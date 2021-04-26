//! Request/Reply (I ask, you answer) example.
//!
//! Request/Reply is used for synchronous communication where each question if responded with a
//! single answer, for example remote procedure calls (RPCs). Like Pipeline, it can also perform
//! load-balancing. This is the only reliable messaging patter in the suite, as it automatically
//! will retry if a request is not matched with a response.
//!
//! This example was derived from [this NNG example][1].
//!
//! [1]: https://nanomsg.org/gettingstarted/nng/reqrep.html
use nng::{Error, Protocol, Socket};
use std::{convert::TryInto, env, process, time::SystemTime};

/// Message representing a date request
const DATE_REQUEST: u64 = 1;

/// Entry point of the application
fn main() -> Result<(), Error> {
    // Begin by parsing the arguments to gather whether this is the request or
    // the reply and what URL to connect with.
    let args: Vec<_> = env::args().take(3).collect();

    match &args[..] {
        [_, t, url] if t == "req" => request(url),
        [_, t, url] if t == "rep" => reply(url),
        _ => {
            println!("Usage: reqrep req|rep <URL>");
            process::exit(1);
        }
    }
}

/// Run the request portion of the program.
fn request(url: &str) -> Result<(), Error> {
    let s = Socket::new(Protocol::Req0)?;
    s.dial(url)?;

    println!("REQUEST: SENDING DATE REQUEST");
    s.send(DATE_REQUEST.to_le_bytes())?;

    println!("REQUEST: WAITING FOR RESPONSE");
    let msg = s.recv()?;
    let epoch = u64::from_le_bytes(msg[..].try_into().unwrap());

    println!("REQUEST: UNIX EPOCH WAS {} SECONDS AGO", epoch);

    Ok(())
}

/// Run the reply portion of the program.
fn reply(url: &str) -> Result<(), Error> {
    let s = Socket::new(Protocol::Rep0)?;
    s.listen(url)?;

    loop {
        println!("REPLY: WAITING FOR COMMAND");
        let mut msg = s.recv()?;

        let cmd = u64::from_le_bytes(msg[..].try_into().unwrap());
        if cmd != DATE_REQUEST {
            println!("REPLY: UNKNOWN COMMAND");
            continue;
        }

        println!("REPLY: RECEIVED DATE REQUEST");
        let rep = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("current system time is before Unix epoch")
            .as_secs();

        msg.clear();
        msg.push_back(&rep.to_le_bytes());

        println!("REPLY: SENDING {}", rep);
        s.send(msg)?;
    }
}
