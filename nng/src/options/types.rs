//! Types of options available.
use std::time::Duration;
use crate::addr::SocketAddr;

create_option!{
	/// The local address used for communication.
	///
	/// The availability of this option is dependent on the transport. Dialers only
	/// have this available when using the `IPC` transport. Listeners have it
	/// available for all transports _except_ `InProc` and `WebSocket`.
	LocalAddr -> SocketAddr:
	Get s = s.getopt_sockaddr(nng_sys::NNG_OPT_LOCADDR);
	Set _s _v = panic!("NNG_OPT_LOCADDR is a read-only option");
}

create_option!{
	/// Whether or not the socket is in "raw" mode.
	///
	/// Raw mode sockets generally do not have any protocol-specific semantics
	/// applied to them; instead the application is expected to perform such
	/// semantics itself. (For example, in “cooked” mode a _rep_ socket would
	/// automatically copy message headers from a received message to the
	/// corresponding reply, whereas in “raw” mode this is not done.)
	///
	/// See [raw mode][1] for more details.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng.7.html#raw_mode
	Raw -> bool:
	Get s = s.getopt_bool(nng_sys::NNG_OPT_RAW);
	Set _s _v = panic!("NNG_OPT_RAW is a read-only option");
}

create_option!{
	/// The minimum amount of time to wait before attempting to establish a
	/// connection after a previous attempt has failed.
	///
	/// If set on a `Socket`, this value becomes the default for new dialers.
	/// Individual dialers can then override the setting.
	ReconnectMinTime -> Option<Duration>:
	Get s = s.getopt_ms(nng_sys::NNG_OPT_RECONNMINT);
	Set s val = s.setopt_ms(nng_sys::NNG_OPT_RECONNMINT, val);
}

create_option!{
	/// The maximum amount of time to wait before attempting to establish a
	/// connection after a previous attempt has failed.
	///
	/// If this is non-zero, then the time between successive connection
	/// attempts will start at the value of `ReconnectMinTime`, and grow
	/// exponentially, until it reaches this value. If this value is zero, then
	/// no exponential back-off between connection attempts is done, and each
	/// attempt will wait the time specified by `ReconnectMinTime`. This can be
	/// set on a socket, but it can also be overridden on an individual dialer.
	ReconnectMaxTime -> Option<Duration>:
	Get s = s.getopt_ms(nng_sys::NNG_OPT_RECONNMAXT);
	Set s val = s.setopt_ms(nng_sys::NNG_OPT_RECONNMAXT, val);
}
