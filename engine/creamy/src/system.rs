use alloc::boxed::Box;
use core::num::NonZeroU8;

use creamy_engine_core::bus::core::{SubscriberId, buffer::DynOutBuf};
use creamy_sdk::{
    Sender,
    api::Plugin,
    stream::{StreamId, StreamMessage, StreamReader},
};
use hashbrown::HashMap;
use smol_str::SmolStr;

use crate::generated::{
    dispatcher::{Dispatcher, MessageHandler},
    system::builtin::{
        ConsumerAppeared, ConsumerDisappeared, GroupDeclared, GroupUndeclared, Log, LogType,
        StreamCancel, StreamKeepAlive,
    },
};

pub struct System<const MS: usize> {
    out: DynOutBuf,
    logs: HashMap<StreamId, StreamReader<Log>, rustc_hash::FxBuildHasher>,

    names: Box<[Option<SmolStr>; MS]>,
    plugins: u8,
}

impl<const MS: usize> Plugin for System<MS> {
    type CustomData = DynOutBuf;
    type Delegate = Dispatcher<Self>;

    fn init(_: Sender, out: Self::CustomData) -> Option<Self> {
        let mut buffer: Box<[Option<SmolStr>; MS]> = unsafe { Box::new_zeroed().assume_init() };
        buffer.fill(None);

        Some(Self {
            out,
            logs: HashMap::default(),
            names: buffer,
            plugins: 1, // System plugin itself
        })
    }
}

impl<const MS: usize> System<MS> {
    const ARRAY_OFFSET: usize = 2;

    pub fn get_plugin_name(&self, id: SubscriberId) -> Option<&str> {
        self.names[id.as_usize() - Self::ARRAY_OFFSET]
            .as_ref()
            .map(SmolStr::as_str)
    }

    pub fn add_plugin_name(&mut self, id: SubscriberId, name: impl Into<SmolStr>) {
        // Subtract 2 because 0 is invalid and we omit the system plugin itself.
        self.names[id.as_usize() - Self::ARRAY_OFFSET] = Some(name.into());
        self.plugins += 1;
    }

    pub fn remove_plugin_name(&mut self, id: SubscriberId) {
        self.names[id.as_usize() - Self::ARRAY_OFFSET] = None;
        self.plugins -= 1;
    }

    pub fn send_group_declared(
        &mut self,
        provider: SubscriberId,
        group_id: NonZeroU8,
        consumer: SubscriberId,
    ) {
        assert!(
            self.out.send(
                &GroupDeclared::PREPARED
                    .with_dst(consumer)
                    .with_provider(provider.as_u8())
                    .with_group_id(group_id.get())
            )
        );
    }

    pub fn send_group_undeclared(
        &mut self,
        provider: SubscriberId,
        group_id: NonZeroU8,
        consumer: SubscriberId,
    ) {
        assert!(
            self.out.send(
                &GroupUndeclared::PREPARED
                    .with_dst(consumer)
                    .with_group_id(group_id.get())
                    .with_provider(provider.as_u8())
            )
        );
    }

    pub fn send_consumer_appeared(
        &mut self,
        provider: SubscriberId,
        group_id: NonZeroU8,
        consumer: SubscriberId,
    ) {
        assert!(
            self.out.send(
                &ConsumerAppeared::PREPARED
                    .with_dst(provider)
                    .with_group_id(group_id.get())
                    .with_consumer_id(consumer.as_u8())
            )
        );
    }

    pub fn send_consumer_disappeared(
        &mut self,
        provider: SubscriberId,
        group_id: NonZeroU8,
        consumer: SubscriberId,
    ) {
        assert!(
            self.out.send(
                &ConsumerDisappeared::PREPARED
                    .with_dst(provider)
                    .with_group_id(group_id.get())
                    .with_consumer_id(consumer.as_u8())
            )
        );
    }
}

impl<const MS: usize> MessageHandler for System<MS> {
    fn handle_stream_keep_alive(&mut self, _: StreamKeepAlive) {}
    fn handle_stream_cancel(&mut self, _: StreamCancel) {}

    fn handle_log(&mut self, message: Log) {
        let stream = self.logs.entry(message.stream_id()).or_default();

        if let Some(result) = stream.accept_message(&message) {
            match result {
                Ok(mut converter) => {
                    self.logs.remove(&message.stream_id()).unwrap();
                    let data = converter.read();
                    let content = str::from_utf8(data.read_content()).unwrap();
                    let target = self.names[message.src.as_usize() - Self::ARRAY_OFFSET]
                        .as_ref()
                        .unwrap();

                    match data.log_type {
                        LogType::Debug => log::debug!(target: target, "{content}"),
                        LogType::Info => log::info!(target: target, "{content}"),
                        LogType::Warning => log::warn!(target:target, "{content}"),
                        LogType::Error => log::error!(target: target, "{content}"),
                    }
                }
                //TODO: don't tick the stream if an error is happened
                Err(err) => log::error!("{err}"),
            }
        }
    }
}
