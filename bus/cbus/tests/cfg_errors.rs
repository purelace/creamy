mod general;
use cbus::{
    BusError,
    config::{BusConfig, ValidConfig},
    define_bus_config,
};

define_bus_config! {
    ZeroMessagesConfig,
    max_subscribers: 32,
    max_messages: 0,
    max_groups: 32,
}

#[test]
fn value_too_small_max_messages() {
    let result = ZeroMessagesConfig.into_valid();
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        BusError::ValueTooSmall {
            name: "max_messages",
            current: 0,
            min: 1024
        }
    );
}

define_bus_config! {
    OneMessageConfig,
    max_subscribers: 32,
    max_messages: 1025,
    max_groups: 32,
}

#[test]
fn value_is_not_multiple_of_2_max_messages() {
    let result = OneMessageConfig.into_valid();
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        BusError::ValueIsNotMultipleOf2 {
            name: "max_messages"
        }
    );
}

define_bus_config! {
    ZeroSubscribersConfig,
    max_subscribers: 0,
    max_messages: 1024,
    max_groups: 32,
}

#[test]
fn value_too_small_max_subscribers() {
    let result = ZeroSubscribersConfig.into_valid();
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        BusError::ValueTooSmall {
            name: "max_subscribers",
            current: 0,
            min: 2
        }
    );
}

define_bus_config! {
    OneSubscriberConfig,
    max_subscribers: 1,
    max_messages: 1024,
    max_groups: 32,
}

#[test]
fn value_is_not_multiple_of_2_max_subscribers() {
    let result = OneSubscriberConfig.into_valid();
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        BusError::ValueIsNotMultipleOf2 {
            name: "max_subscribers"
        }
    );
}

define_bus_config! {
    ZeroGroupsConfig,
    max_subscribers: 32,
    max_messages: 1024,
    max_groups: 0,
}

#[test]
fn groups_is_not_power_of_two() {
    let result = ZeroGroupsConfig.into_valid();
    assert!(result.is_err());
    assert_eq!(result.err().unwrap(), BusError::GroupsIsNotPowerOf2);
}

define_bus_config! {
    MessagesValueTooBigConfigV1,
    max_subscribers: 32,
    max_messages: usize::MAX / 2 + 1,
    max_groups: 32,
}

define_bus_config! {
    MessagesValueTooBigConfigV2,
    max_subscribers: 128,
    max_messages: (usize::MAX / 2 + 1) / 32,
    max_groups: 128,
}

define_bus_config! {
    MessagesValueTooBigConfigV3,
    max_subscribers: 32,
    max_messages: (usize::MAX / 2 + 1) / 32,
    max_groups: 32,
}

#[test]
fn max_message_value_too_big() {
    max_messages_value_too_big_generic(MessagesValueTooBigConfigV1);
    max_messages_value_too_big_generic(MessagesValueTooBigConfigV2);
    max_messages_value_too_big_generic(MessagesValueTooBigConfigV3);
}

fn max_messages_value_too_big_generic<C: BusConfig>(config: C) {
    let result = config.into_valid();
    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        BusError::ValueTooBig {
            name: "max_messages"
        }
    );
}
