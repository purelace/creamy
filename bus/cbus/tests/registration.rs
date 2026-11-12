mod general;

use std::collections::HashMap;

use cbus::{
    BusDriver, BusError, DataIterator, SubscriberLookupData,
    core::{Subscriber, SubscriberId, UntypedMessage},
    define_bus,
};

use crate::general::{EmptyDriver, EmptySubscriber};

define_bus! {
    max_messages: 1024,
    max_protocols: 32,
    max_subscribers: 32,
}

#[derive(Debug)]
struct DnsSubscriber;
impl Subscriber for DnsSubscriber {
    fn notify(&mut self) {}
}

#[derive(Debug)]
struct Dns {
    table: HashMap<u8, &'static str>,
    _incoming: IncBuf,
    outgoing: OutBuf,
}

impl Dns {
    pub fn new(incoming: IncBuf, outgoing: OutBuf) -> Self {
        let mut table = HashMap::new();
        table.insert(2, "protocol_0");
        table.insert(3, "protocol_1");
        table.insert(4, "protocol_2");
        table.insert(5, "protocol_3");
        table.insert(6, "protocol_4");
        table.insert(7, "protocol_5");
        table.insert(8, "protocol_6");
        table.insert(9, "protocol_7");

        Self {
            table,
            _incoming: incoming,
            outgoing,
        }
    }
}

impl BusDriver for Dns {
    fn on_subscribe(&mut self, id: SubscriberId) -> impl DataIterator {
        if id.as_u8() == 1 {
            return std::iter::once(SubscriberLookupData {
                consumer_protocol_id: 1,
                provider_protocol_id: 1,
                provider_id: SubscriberId::new_u8(1).unwrap(),
            });
        }

        let mut message = UntypedMessage {
            dst: id.as_u8(),
            protocol: 1,
            src: 0,
            kind: 0,
            payload: [0; 28],
        };

        let ident = self.table.get(&id.as_u8()).unwrap();
        let bytes = ident.as_bytes();
        message.payload[0] = bytes[0];
        message.payload[1] = bytes[1];
        message.payload[2] = bytes[2];
        message.payload[3] = bytes[3];
        message.payload[4] = bytes[4];
        message.payload[5] = bytes[5];
        message.payload[6] = bytes[6];
        message.payload[7] = bytes[7];
        message.payload[8] = bytes[8];
        message.payload[9] = bytes[9];

        assert!(self.outgoing.send_many_iter_exact(std::iter::once(message)));

        std::iter::once(SubscriberLookupData {
            consumer_protocol_id: 1,
            provider_protocol_id: 1,
            provider_id: SubscriberId::new_u8(1).unwrap(),
        })
    }

    fn on_unsubscribe(&mut self, _: SubscriberId) {}
}

#[derive(Debug)]
struct Resolver {
    inc: IncBuf,
    _out: OutBuf,
    get_ident: bool,
    ident: &'static str,
}

impl Resolver {
    const fn new(inc: IncBuf, out: OutBuf, ident: &'static str) -> Self {
        Self {
            inc,
            _out: out,
            get_ident: false,
            ident,
        }
    }
}

impl Subscriber for Resolver {
    fn notify(&mut self) {
        let Some(message) = self.inc.pop() else {
            return;
        };

        let ident = str::from_utf8(&message.payload[..10]).unwrap();
        assert_eq!(ident, self.ident);

        self.get_ident = true;

        self.inc.clear();
    }
}

#[derive(Debug)]
enum Subscribers {
    Dns(DnsSubscriber),
    Resolver(Resolver),
}

impl Subscriber for Subscribers {
    fn notify(&mut self) {
        match self {
            Subscribers::Dns(sub) => sub.notify(),
            Subscribers::Resolver(res) => res.notify(),
        }
    }
}

impl Subscribers {
    fn resolver(&self) -> &Resolver {
        match self {
            Subscribers::Resolver(res) => res,
            Subscribers::Dns(_) => unreachable!(),
        }
    }
}

#[test]
fn register_and_unregister() -> Result<(), Box<dyn std::error::Error>> {
    let inc = IncBuf::default();
    let out = OutBuf::default();

    let mut bus = MessageBus::<Dns, Subscribers>::new(Dns::new(inc.clone(), out.clone()));

    bus.add_subscriber_with(inc, out, Subscribers::Dns(DnsSubscriber))?;

    macro_rules! add {
        ($ident: expr) => {{ bus.add_subscriber(|inc, out| Subscribers::Resolver(Resolver::new(inc, out, $ident)))? }};
    }

    let id0 = add!("protocol_0");
    let id1 = add!("protocol_1");
    let id2 = add!("protocol_2");
    let id3 = add!("protocol_3");
    let id4 = add!("protocol_4");
    let id5 = add!("protocol_5");
    let id6 = add!("protocol_6");
    let id7 = add!("protocol_7");

    bus.full_tick();

    assert!(bus.remove_subscriber(id0)?.resolver().get_ident);
    assert!(bus.remove_subscriber(id1)?.resolver().get_ident);
    assert!(bus.remove_subscriber(id2)?.resolver().get_ident);
    assert!(bus.remove_subscriber(id3)?.resolver().get_ident);
    assert!(bus.remove_subscriber(id4)?.resolver().get_ident);
    assert!(bus.remove_subscriber(id5)?.resolver().get_ident);
    assert!(bus.remove_subscriber(id6)?.resolver().get_ident);
    assert!(bus.remove_subscriber(id7)?.resolver().get_ident);

    assert_eq!(bus.subscribers(), 1);

    Ok(())
}

#[test]
fn bus_exceed_error() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = MessageBus::<EmptyDriver, EmptySubscriber<MAX_MESSAGES>>::new(EmptyDriver);
    assert_eq!(bus.subscribers(), 0);

    for _ in 1..MAX_SUBSCRIBERS {
        bus.add_subscriber(EmptySubscriber::new)?;
    }
    assert_eq!(bus.subscribers(), MAX_SUBSCRIBERS as u8 - 1);

    assert_eq!(
        bus.add_subscriber(EmptySubscriber::new),
        Err(BusError::PoolExhausted {
            max: MAX_SUBSCRIBERS as u8
        })
    );

    Ok(())
}

#[test]
fn subscriber_remove() {
    let mut bus = MessageBus::<EmptyDriver, EmptySubscriber<MAX_MESSAGES>>::new(EmptyDriver);

    let ids = (1..MAX_SUBSCRIBERS)
        .map(|_| bus.add_subscriber(EmptySubscriber::new).unwrap())
        .collect::<Vec<_>>();

    bus.full_tick();

    for id in ids {
        assert!(bus.remove_subscriber(id).is_ok());
    }

    assert_eq!(bus.subscribers(), 0);
}

/*
#[test]
fn send_remove_request_twice() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus =
        MessageBus::<Legacy, EmptyDriver, MAX_MESSAGES, EmptySubscriber>::new(EmptyDriver);
    let id = bus.add_subscriber(IncBuf::new(), OutBuf::new(), EmptySubscriber)?;

    assert!(bus.remove_subscriber(id).is_ok());
    let result = bus.remove_subscriber(id);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), BusError::RequestAlreadySent);

    Ok(())
}*/

#[test]
fn remove_subscriber_with_incorrect_id() {
    let mut bus = MessageBus::<EmptyDriver, EmptySubscriber<MAX_MESSAGES>>::new(EmptyDriver);
    let result = bus.remove_subscriber(
        // SAFETY: 31 != 0
        unsafe { SubscriberId::new_unchecked(31) },
    );
    assert_eq!(result, Err(BusError::SubscriberNotRegistered));
}
