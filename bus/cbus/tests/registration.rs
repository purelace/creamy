mod general;

use std::collections::HashMap;

use cbus::{
    BusDriver, BusError, DataIterator, DriverAnswer, MessageBus, OldDataIterator,
    SubscriberLookupData, SubscriberOldLookupData,
    config::{BusConfig, Legacy, ValidConfig},
};
use cbus_core::{
    Subscriber, SubscriberId, UntypedMessage,
    buffer::{Incoming, Outgoing},
};

use crate::general::{EmptyDriver, EmptySubscriber};

struct Dns {
    table: HashMap<u8, &'static str>,
    outgoing: Outgoing,
}

impl Dns {
    pub fn new<C: BusConfig>(_config: &ValidConfig<C>, outgoing: Outgoing) -> Self {
        let mut table = HashMap::new();
        table.insert(1, "group_0");
        table.insert(2, "group_1");
        table.insert(3, "group_2");
        table.insert(4, "group_3");
        table.insert(5, "group_4");
        table.insert(6, "group_5");
        table.insert(7, "group_6");
        table.insert(8, "group_7");

        Self { table, outgoing }
    }
}

impl BusDriver for Dns {
    fn on_subscribe(&mut self, id: u8) -> DriverAnswer<impl DataIterator, impl DataIterator> {
        let mut message = UntypedMessage {
            dst: id,
            group: id,
            src: 0,
            kind: 0,
            version: 0,
            payload: [0; 27],
        };

        let ident = self.table.get(&id).unwrap();
        let bytes = ident.as_bytes();
        message.payload[0] = bytes[0];
        message.payload[1] = bytes[1];
        message.payload[2] = bytes[2];
        message.payload[3] = bytes[3];
        message.payload[4] = bytes[4];
        message.payload[5] = bytes[5];
        message.payload[6] = bytes[6];

        assert!(self.outgoing.send_many_iter_exact(std::iter::once(message)));

        let subscriber_changes = std::iter::once(SubscriberLookupData {
            local_group_id: 1,
            global_group_id: 1,
        });
        let driver_changes = std::iter::once(SubscriberLookupData {
            local_group_id: id as usize,
            global_group_id: id,
        });

        DriverAnswer {
            subscriber_changes,
            driver_changes,
        }
    }

    fn on_unsubscribe(
        &mut self,
        id: u8,
    ) -> DriverAnswer<impl OldDataIterator, impl OldDataIterator> {
        let subscriber_changes = std::iter::once(SubscriberOldLookupData {
            global_group_id: id,
        });

        let driver_changes = std::iter::once(SubscriberOldLookupData {
            global_group_id: id,
        });

        DriverAnswer {
            subscriber_changes,
            driver_changes,
        }
    }
}

struct Resolver {
    inc: Incoming,
    _out: Outgoing,
    get_ident: bool,
    ident: &'static str,
}

impl Resolver {
    const fn new(inc: Incoming, out: Outgoing, ident: &'static str) -> Self {
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
        //let slice = self.inc.as_slice();
        let slice = self.inc.pop_all();

        if slice.is_empty() {
            return;
        }

        let message = slice[0];
        let ident = str::from_utf8(&message.payload[..7]).unwrap();
        assert_eq!(ident, self.ident);

        self.get_ident = true;

        self.inc.clear();
    }
}

#[test]
fn register_and_unregister() {
    let mut bus = MessageBus::<Dns, Resolver>::new(Legacy, Dns::new).unwrap();
    let id0 = bus
        .add_subscriber(|inc, out| Resolver::new(inc, out, "group_0"))
        .unwrap();
    let id1 = bus
        .add_subscriber(|inc, out| Resolver::new(inc, out, "group_1"))
        .unwrap();
    let id2 = bus
        .add_subscriber(|inc, out| Resolver::new(inc, out, "group_2"))
        .unwrap();
    let id3 = bus
        .add_subscriber(|inc, out| Resolver::new(inc, out, "group_3"))
        .unwrap();
    let id4 = bus
        .add_subscriber(|inc, out| Resolver::new(inc, out, "group_4"))
        .unwrap();
    let id5 = bus
        .add_subscriber(|inc, out| Resolver::new(inc, out, "group_5"))
        .unwrap();
    let id6 = bus
        .add_subscriber(|inc, out| Resolver::new(inc, out, "group_6"))
        .unwrap();
    let id7 = bus
        .add_subscriber(|inc, out| Resolver::new(inc, out, "group_7"))
        .unwrap();

    bus.full_tick();

    bus.send_remove_request(id0).unwrap();
    bus.send_remove_request(id1).unwrap();
    bus.send_remove_request(id2).unwrap();
    bus.send_remove_request(id3).unwrap();
    bus.send_remove_request(id4).unwrap();
    bus.send_remove_request(id5).unwrap();
    bus.send_remove_request(id6).unwrap();
    bus.send_remove_request(id7).unwrap();

    bus.full_tick();

    let removed = bus.removed();
    assert!(removed[0].get_ident);
    assert!(removed[1].get_ident);
    assert!(removed[2].get_ident);
    assert!(removed[3].get_ident);
    assert!(removed[4].get_ident);
    assert!(removed[5].get_ident);
    assert!(removed[6].get_ident);
    assert!(removed[7].get_ident);
}

#[test]
fn bus_exceed_error() {
    let mut bus =
        MessageBus::<EmptyDriver, EmptySubscriber>::new(Legacy, EmptyDriver::new).unwrap();
    assert_eq!(bus.subscribers(), 1);
    let max_subs = Legacy.max_subscribers() - 1;
    for _ in 0..max_subs {
        bus.add_subscriber(|_, _| EmptySubscriber).unwrap();
    }
    assert_eq!(bus.subscribers(), Legacy.max_subscribers() as usize);
    let result = bus.add_subscriber(|_, _| EmptySubscriber);
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        BusError::PoolExhausted {
            max: Legacy.max_subscribers() as usize
        }
    )
}

#[test]
fn subscriber_remove() {
    let mut bus =
        MessageBus::<EmptyDriver, EmptySubscriber>::new(Legacy, EmptyDriver::new).unwrap();

    let max_subs = Legacy.max_subscribers() - 1;

    let ids = (0..max_subs)
        .map(|_| bus.add_subscriber(|_, _| EmptySubscriber).unwrap())
        .collect::<Vec<_>>();

    bus.full_tick();

    for id in ids {
        assert!(bus.send_remove_request(id).is_ok());
    }

    bus.full_tick();

    let removed = bus.removed();
    assert_eq!(removed.len(), max_subs as usize);
}

#[test]
fn remove_zero() {
    let mut bus =
        MessageBus::<EmptyDriver, EmptySubscriber>::new(Legacy, EmptyDriver::new).unwrap();
    let result = bus.send_remove_request(SubscriberId::new(0));
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), BusError::InvalidSubscriberId);
}

#[test]
fn send_remove_request_twice() {
    let mut bus =
        MessageBus::<EmptyDriver, EmptySubscriber>::new(Legacy, EmptyDriver::new).unwrap();
    let id = bus.add_subscriber(|_, _| EmptySubscriber).unwrap();

    assert!(bus.send_remove_request(id).is_ok());
    let result = bus.send_remove_request(id);
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), BusError::RequestAlreadySent);
}

#[test]
fn send_remove_request_with_incorrect_id() {
    let mut bus =
        MessageBus::<EmptyDriver, EmptySubscriber>::new(Legacy, EmptyDriver::new).unwrap();
    let result = bus.send_remove_request(SubscriberId::new(128));
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), BusError::SubscriberNotRegistered);
}
