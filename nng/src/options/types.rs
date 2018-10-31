//! Types of options available.
use std::time::Duration;
use crate::addr::SocketAddr;

create_option!{
	/// The local address used for communication.
	///
	/// The availability of this option is dependent on the transport. Dialers only
	/// have this available when using the `IPC` transport. Listeners have it
	/// available for all transports _except_ `InProc` and `WebSocket`.
	///
	/// ## Support
	///
	/// * Dialers can read from this with the IPC transport.
	/// * Listeners can read from this on the following transports:
	///     * TCP
	///     * ZeroTier
	///     * IPC
	///     * TLS
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
	/// ## Support
	///
	/// * Sockets can read this option.
	/// * Dialers and Listeners can retrieve this from their owning Socket.
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
	///
	/// ## Support
	///
	/// * Dialers can use this option.
	/// * Sockets can use this option to create a new default value.
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
	///
	/// ## Support
	///
	/// * Dialers can use this option.
	/// * Sockets can use this option to create a new default value.
	ReconnectMaxTime -> Option<Duration>:
	Get s = s.getopt_ms(nng_sys::NNG_OPT_RECONNMAXT);
	Set s val = s.setopt_ms(nng_sys::NNG_OPT_RECONNMAXT, val);
}

create_option!{
	/// The depth of the socket's receive buffer as a number of messages.
	///
	/// Messages received by the transport may be buffered until the
	/// application has accepted them for delivery.
	///
	/// ## Support
	///
	/// * Sockets can read and write this option.
	/// * Dialers and Listeners can retrieve it from their owning Socket.
	RecvBufferSize -> i32:
	Get s = s.getopt_int(nng_sys::NNG_OPT_RECVBUF);
	Set s val = s.setopt_int(nng_sys::NNG_OPT_RECVBUF, val);
}

create_option!{
	/// The maximum message size that the will be accepted from a remote peer.
	///
	/// If a peer attempts to send a message larger than this, then the message
	/// will be discarded. If the value of this is zero, then no limit on
	/// message sizes is enforced. This option exists to prevent certain kinds
	/// of denial-of-service attacks, where a malicious agent can claim to want
	/// to send an extraordinarily large message, without sending any data.
	/// This option can be set for the socket, but may be overridden for on a
	/// per-dialer or per-listener basis.
	///
	/// Note that some transports may have further message size restrictions.
	///
	/// ## Support
	///
	/// * Dialers and Listeners can use this with the following transports:
	///     * TCP
	///     * ZeroTier
	///     * IPC
	///     * TLS
	///     * WebSocket
	/// * Sockets can utilize this to set a new default value.
	RecvMaxSize -> usize:
	Get s = s.getopt_size(nng_sys::NNG_OPT_RECVMAXSZ);
	Set s val = s.setopt_size(nng_sys::NNG_OPT_RECVMAXSZ, val);
}

create_option!{
	/// The socket receive timeout.
	///
	/// When no message is available for receiving at the socket for this period
	/// of time, receive operations will fail with `ErrorKind::TimedOut`.
	///
	/// ## Support
	///
	/// * Sockets can utilize this value.
	/// * Dialers and Listeners can retrieve it from their owning Socket.
	RecvTimeout -> Option<Duration>:
	Get s = s.getopt_ms(nng_sys::NNG_OPT_RECVTIMEO);
	Set s val = s.setopt_ms(nng_sys::NNG_OPT_RECVTIMEO, val);
}

create_option!{
	/// The depth of the socket send buffer as a number of messages.
	///
	/// Messages sent by an application may be buffered by the socket until a
	/// transport is ready to accept them for delivery. This value must be an
	/// integer between 0 and 8192, inclusive.
	///
	/// ## Support
	///
	/// * Sockets can utilize this value.
	/// * Dialers and Listeners can retrieve it from their owning Socket.
	SendBufferSize -> i32:
	Get s = s.getopt_int(nng_sys::NNG_OPT_SENDBUF);
	Set s val = s.setopt_int(nng_sys::NNG_OPT_SENDBUF, val);
}

create_option!{
	/// The socket send timeout.
	///
	/// When a message cannot be queued for delivery by the socket for this
	/// period of time (such as if send buffers are full), the operation will
	/// fail with `ErrorKind::TimedOut`.
	///
	/// ## Support
	///
	/// * Sockets can utilize this value.
	/// * Dialers and Listeners can retrieve it from their owning Socket.
	SendTimeout -> Option<Duration>:
	Get s = s.getopt_ms(nng_sys::NNG_OPT_SENDTIMEO);
	Set s val = s.setopt_ms(nng_sys::NNG_OPT_SENDTIMEO, val);
}

create_option!{
	/// The socket name.
	///
	/// By default this is a string corresponding to the value of the socket.
	/// The string must fit within 63-bytes but it can be changed for other
	/// application uses.
	///
	/// ## Support
	///
	/// * Sockets can utilize this value.
	/// * Dialers and Listeners can retrieve it from their owning Socket.
	SocketName -> String:
	Get s = s.getopt_string(nng_sys::NNG_OPT_SOCKNAME);
	Set s val = s.setopt_string(nng_sys::NNG_OPT_SOCKNAME, &val);
}

create_option!{
	/// The maximum number of "hops" a message may traverse.
	///
	/// The intention here is to prevent forwarding loops in [device chains][1].
	/// Note that not all protocols support this option and those that do
	/// generally have a default value of 8.
	///
	/// Each node along a forwarding path may have its own value for the
	/// maximum time-to-live, and performs its own checks before forwarding a
	/// message. Therefore it is helpful if all nodes in the topology use the
	/// same value for this option.
	///
	/// ## Support
	///
	/// * Sockets can use this with the following protocols:
	///     * Pair v1
	///     * Rep v0
	///     * Req v0
	///     * Surveyor v0
	///     * Respondent v0
	/// * Dialers and Listeners can retrieve it from their owning Socket, if applicable.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_device.3.html
	MaxTtl -> u8:
	Get s = s.getopt_int(nng_sys::NNG_OPT_MAXTTL).map(|v| v as u8);
	Set s val = s.setopt_int(nng_sys::NNG_OPT_MAXTTL, val as i32);
}

create_option!{
	/// The URL with which a listener or dialer was configured.
	///
	/// Note that some transports will canonify URLs before returning them to
	/// the application.
	///
	/// ## Support
	///
	/// * Dialers and Listeners can read this value.
	Url -> String:
	Get s = s.getopt_string(nng_sys::NNG_OPT_URL);
	Set _s _v = panic!("URL is a read-only option");
}

/// Options relating to the socket protocol.
pub mod protocol
{
	/// Options dealing with the PUBSUB protocol.
	pub mod pubsub
	{
		create_option!{
			/// Register a topic that the subscriber is interested in.
			///
			/// This option takes an array of bytes, of arbitrary size. Each
			/// incoming message is checked against the list of subscribed
			/// topics. If the body begins with the entire set of bytes in the
			/// topic, then the message is accepted. If no topic matches, then
			/// the message is discarded.
			///
			/// To receive all messages, an empty topic (zero length) can be
			/// used. To receive any messages, at least one subscription must
			/// exist.
			///
			/// ## Support
			///
			/// * Sockets can use this option when using the Sub v0 protocol.
			Subscribe -> Vec<u8>:
			Get _s = panic!("Subscribe is a write-only option");
			Set s val = s.setopt(nng_sys::protocol::pubsub0::NNG_OPT_SUB_SUBSCRIBE, &val);
		}

		create_option!{
			/// Remove a topic from the subscription list.
			///
			/// Note that if the topic was not previously subscribed via the
			/// `Subscribe` option, then using this option will result in
			/// `ErrorKind::EntryNotFound`.
			///
			/// ## Support
			///
			/// * Sockets can use this option when using the Sub v0 protocol.
			Unsubscribe -> Vec<u8>:
			Get _s = panic!("Subscribe is a write-only option");
			Set s val = s.setopt(nng_sys::protocol::pubsub0::NNG_OPT_SUB_UNSUBSCRIBE, &val);
		}
	}

	/// Options dealing with the REQREP protocol.
	pub mod reqrep
	{
		use std::time::Duration;

		create_option!{
			/// Amount of time to wait before sending a new request.
			///
			/// When a new request is started, a timer of this duration is also
			/// started. If no reply is received before this timer expires,
			/// then the request will be resent. (Requests are also
			/// automatically resent if the peer to whom the original request
			/// was sent disconnects, or if a peer becomes available while the
			/// requester is waiting for an available peer.)
			///
			/// ## Support
			///
			/// * Sockets can read and write this value when using the following protocols:
			///     * Req v0
			/// * Dialers and Listeners can retrieve it from their owning Socket, if applicable.
			ResendTime -> Option<Duration>:
			Get s = s.getopt_ms(nng_sys::protocol::reqrep0::NNG_OPT_REQ_RESENDTIME);
			Set s val = s.setopt_ms(nng_sys::protocol::reqrep0::NNG_OPT_REQ_RESENDTIME, val);
		}
	}

	/// Options dealing with the survey protocol.
	pub mod survey
	{
		use std::time::Duration;

		create_option!{
			/// Amount of time that the following surveys will last.
			///
			/// When a new survey is started, a timer of this duration is also
			/// started. Any responses arriving this time will be discarded.
			/// Attempts to receive after the timer expires with no other
			/// surveys started will result in `ErrorKind::IncorrectState`.
			/// Attempts to receive when this timer expires will result in
			/// `ErrorKind::TimedOut`.
			///
			/// ## Support
			///
			/// * Sockets can read and write this value when using the following protocols:
			///     * Surveyor v0
			/// * Dialers and Listeners can retrieve it from their owning Socket, if applicable.
			SurveyTime -> Option<Duration>:
			Get s = s.getopt_ms(nng_sys::protocol::survey0::NNG_OPT_SURVEYOR_SURVEYTIME);
			Set s val = s.setopt_ms(nng_sys::protocol::survey0::NNG_OPT_SURVEYOR_SURVEYTIME, val);
		}
	}
}

/// Options dealing with the underlying transport.
pub mod transport
{
	/// Options related to transports built on top of TCP.
	pub mod tcp
	{
		create_option!{
			/// Disable (or enable) the use of Nagle's algorithm for TCP
			/// connections.
			///
			/// When `true` (the default), messages are sent immediately by the
			/// underlying TCP stream without waiting to gather more data. When
			/// `false`, Nagle's algorithm is enabled, and the TCP stream may wait
			/// briefly in attempt to coalesce messages. Nagle's algorithm is
			/// useful on low-bandwidth connections to reduce overhead, but it
			/// comes at a cost to latency.
			///
			/// ## Support
			///
			/// * Dialers and Listeners can use this option with the following transports:
			///     * TCP
			///     * TLS
			/// * Sockets can use this to set a default value.
			NoDelay -> bool:
			Get s = s.getopt_bool(nng_sys::NNG_OPT_TCP_NODELAY);
			Set s val = s.setopt_bool(nng_sys::NNG_OPT_TCP_NODELAY, val);
		}

		create_option!{
			/// Enable the sending of keep-alive messages on the underlying TCP stream.
			///
			/// This option is `false` by default. When enabled, if no messages are
			/// seen for a period of time, then a zero length TCP message is sent
			/// with the ACK flag set in an attempt to tickle some traffic from the
			/// peer. If none is still seen (after some platform-specific number of
			/// retries and timeouts), then the remote peer is presumed dead, and
			/// the connection is closed.
			///
			/// This option has two purposes. First, it can be used to detect dead
			/// peers on an otherwise quiescent network. Second, it can be used to
			/// keep connection table entries in NAT and other middleware from
			/// being expiring due to lack of activity. 
			///
			/// ## Support
			///
			/// * Dialers and Listeners can use this option with the following transports:
			///     * TCP
			///     * TLS
			/// * Sockets can use this to set a default value.
			KeepAlive -> bool:
			Get s = s.getopt_bool(nng_sys::NNG_OPT_TCP_KEEPALIVE);
			Set s val = s.setopt_bool(nng_sys::NNG_OPT_TCP_KEEPALIVE, val);
		}
	}

	/// Options related to the TLS transport.
	pub mod tls
	{
		create_option!{
			/// Used to load certificates associated associated private key from a
			/// file.
			///
			/// See the [CA Config][1] documentation for more information.
			///
			/// ## Support
			///
			/// * Dialers and Listeners can use this option with the following transports:
			///     * TLS
			///     * WebSocket (Secure)
			/// * Sockets can use this to set a default value.
			///
			/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_tls.7.html
			CaFile -> String:
			Get _s = panic!("CA File is a write-only option");
			Set s val = s.setopt_string(nng_sys::NNG_OPT_TLS_CA_FILE, &val);
		}

		create_option!{
			/// Used to load the local certificate and associated private key from
			/// a file.
			///
			/// The private key used must be unencrypted. See the [nng docs][1] for
			/// more information.
			///
			/// ## Support
			///
			/// * Dialers and Listeners can use this option with the following transports:
			///     * TLS
			///     * WebSocket (Secure)
			/// * Sockets can use this to set a default value.
			///
			/// [1]: https://nanomsg.github.io/nng/man/v1.0.0/nng_tls.7.html
			CertKeyFile -> String:
			Get _s = panic!("Cert Key File is a write-only option");
			Set s val = s.setopt_string(nng_sys::NNG_OPT_TLS_CERT_KEY_FILE, &val);
		}
	}

	/// Options related to the WebSocket and Secure WebSocket transports.
	pub mod websocket
	{
		create_option!{
			/// A multiline string terminated by CRLF sequences, that can be used
			/// to add further headers to the HTTP request sent when connecting.
			///
			/// ## Support
			///
			/// * Dialers can use this when using the WebSocket transport.
			/// * Sockets can use this to set a default value.
			RequestHeaders -> String:
			Get _s = panic!("Request headers are a write-only option");
			Set s val = s.setopt_string(nng_sys::transport::websocket::NNG_OPT_WS_REQUEST_HEADERS, &val);
		}

		create_option!{
			/// A multiline string terminated by CRLF sequences, that can be used
			/// to add further headers to the HTTP response sent when connecting.
			///
			/// ## Support
			///
			/// * Listeners can use this when using the WebSocket transport.
			/// * Sockets can use this to set a default value.
			ResponseHeaders -> String:
			Get _s = panic!("Response headers are a write-only option");
			Set s val = s.setopt_string(nng_sys::transport::websocket::NNG_OPT_WS_RESPONSE_HEADERS, &val);
		}
	}
}
