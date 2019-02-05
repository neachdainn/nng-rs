/// Protocols available for use by sockets.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum Protocol
{
	/// Version 0 of the bus protocol.
	///
	/// The _bus_ protocol provides for building mesh networks where every peer
	/// is connected to every other peer. In this protocol, each message sent
	/// by a node is sent to every one of its directly connected peers. See
	/// the [bus documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_bus.7.html
	Bus0,

	/// Version 0 of the pair protocol.
	///
	/// The _pair_ protocol implements a peer-to-peer pattern, where
	/// relationships between peers are one-to-one. See the
	/// [pair documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_pair.7.html
	Pair0,

	/// Version 1 of the pair protocol.
	///
	/// The _pair_ protocol implements a peer-to-peer pattern, where
	/// relationships between peers are one-to-one. Version 1 of this protocol
	/// supports and optional _polyamorous_ mode. See the [pair
	/// documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_pair.7.html
	Pair1,

	/// Version 0 of the publisher protocol.
	///
	/// The _pub_ protocol is one half of a publisher/subscriber pattern. In
	/// this pattern, a publisher sends data, which is broadcast to all
	/// subscribers. The subscribing applications only see the data to which
	/// they have subscribed. See the [publisher/subscriber documentation][1]
	/// for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_pub.7.html
	Pub0,

	/// Version 0 of the pull protocol.
	///
	/// The _pull_ protocol is one half of a pipeline pattern. The other half
	/// is the _push_ protocol. In the pipeline pattern, pushers distribute
	/// messages to pullers. Each message sent by a pusher will be sent to one
	/// of its peer pullers, chosen in a round-robin fashion from the set of
	/// connected peers available for receiving. This property makes this
	/// pattern useful in load-balancing scenarios.
	///
	/// See the [pipeline documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_pull.7.html
	Pull0,

	/// Version 0 of the push protocol.
	///
	/// The _push_ protocol is one half of a pipeline pattern. The other side
	/// is the _pull_ protocol. In the pipeline pattern, pushers distribute
	/// messages to pullers. Each message sent by a pusher will be sent to one
	/// of its peer pullers, chosen in a round-robin fashion from the set of
	/// connected peers available for receiving. This property makes this
	/// pattern useful in load-balancing scenarios.
	///
	/// See the [pipeline documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_push.7.html
	Push0,

	/// Version 0 of the reply protocol.
	///
	/// The _rep_ protocol is one half of a request/reply pattern. In this
	/// pattern, a requester sends a message to one replier, who is expected to
	/// reply. The request is resent if no reply arrives, until a reply is
	/// received or the request times out.
	///
	/// See the [request/reply documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_rep.7.html
	Rep0,

	/// Version 0 of the request protocol.
	///
	/// The _req_ protocol is one half of a request/reply pattern. In this
	/// pattern, a requester sends a message to one replier, who is expected to
	/// reply. The request is resent if no reply arrives, until a reply is
	/// received or the request times out.
	///
	/// See the [request/reply documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_req.7.html
	Req0,

	/// Version 0 of the respondent protocol.
	///
	/// The _respondent_ protocol is one half of a survey pattern. In this
	/// pattern, a surveyor sends a survey, which is broadcast to all peer
	/// respondents. The respondents then have a chance to reply (but are not
	/// obliged to reply). The survey itself is a timed event, so that
	/// responses received after the survey has finished are discarded.
	///
	/// See the [survery documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_respondent.7.html
	Respondent0,

	/// Version 0 of the subscriber protocol.
	///
	/// The _sub_ protocol is one half of a publisher/subscriber pattern. In
	/// this pattern, a publisher sends data, which is broadcast to all
	/// subscribers. The subscribing applications only see the data to which
	/// they have subscribed.
	///
	/// See the [publisher/subscriber documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_sub.7.html
	Sub0,

	/// Version 0 of the surveyor protocol.
	///
	/// The _surveyor_ protocol is one half of a survey pattern. In this
	/// pattern, a surveyor sends a survey, which is broadcast to all peer
	/// respondents. The respondents then have a chance to reply (but are not
	/// obliged to reply). The survey itself is a timed event, so that
	/// responses received after the survey has finished are discarded.
	///
	/// See the [survey documentation][1] for more information.
	///
	/// [1]: https://nanomsg.github.io/nng/man/v1.1.0/nng_surveyor.7.html
	Surveyor0,
}
