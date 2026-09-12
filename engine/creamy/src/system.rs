use alloc::boxed::Box;
use core::num::NonZeroU8;

use creamy_engine_core::bus::core::{
    Subscriber, SubscriberId, UntypedMessage,
    buffer::{IncBuf, OutBuf, runtime::DynIncBuf},
};
use creamy_sdk::{
    api::{CustomHandler, handle_incoming},
    logging::{LogReader, LogType},
    stream::{StreamId, StreamMessage, StreamReader},
};
use hashbrown::HashMap;
use smol_str::SmolStr;

use crate::generated::{
    dispatcher::{self, MessageHandler},
    system::builtin::{GroupDeclared, Log, StreamCancel, StreamKeepAlive},
};

pub struct SystemPlugin<const S: usize, const M: usize> {
    inc: IncBuf<M>,
    out: OutBuf<M>,
    logs: HashMap<StreamId, StreamReader<LogReader>, rustc_hash::FxBuildHasher>,

    names: Box<[Option<SmolStr>; S]>,
    plugins: u8,
}

impl<const S: usize, const M: usize> SystemPlugin<S, M> {
    const _ASSERT: () = {
        const MAX_SUBSCRIBERS: usize = 255;
        assert!(
            S <= MAX_SUBSCRIBERS,
            "Размер массива S не должен превышать 255!"
        );
    };

    const ARRAY_OFFSET: usize = 2;

    #[must_use]
    pub fn new(inc: IncBuf<M>, out: OutBuf<M>) -> Self {
        let mut buffer: Box<[Option<SmolStr>; S]> = unsafe { Box::new_zeroed().assume_init() };
        for element in buffer.iter_mut() {
            *element = None;
        }

        Self {
            inc,
            out,
            logs: HashMap::default(),
            names: buffer,
            plugins: 1, // System plugin itself
        }
    }

    pub fn get_plugin_name(&self, id: SubscriberId) -> Option<&str> {
        self.names[id.get() as usize - Self::ARRAY_OFFSET]
            .as_ref()
            .map(SmolStr::as_str)
    }

    pub fn add_plugin_name(&mut self, id: SubscriberId, name: impl Into<SmolStr>) {
        // Subtract 2 because 0 is invalid and we omit the system plugin itself.
        self.names[id.get() as usize - Self::ARRAY_OFFSET] = Some(name.into());
        self.plugins += 1;
    }

    pub fn remove_plugin_name(&mut self, id: SubscriberId) {
        self.names[id.get() as usize - Self::ARRAY_OFFSET] = None;
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
                GroupDeclared::PREPARED
                    .with_dst(consumer.get())
                    .with_provider(provider.get())
                    .with_group_id(group_id.get())
            )
        );
    }
}

impl<const S: usize, const M: usize> Subscriber for SystemPlugin<S, M> {
    fn notify(&mut self) {
        unsafe {
            let dyn_buf = self.inc.as_inner_mut().as_dyn_buf();
            let inc_buf = DynIncBuf::from_buf(dyn_buf);
            handle_incoming(self, inc_buf);
        }
    }
}

impl<const S: usize, const M: usize> CustomHandler for SystemPlugin<S, M> {
    fn handle_message(&mut self, dispatch_value: u32, message: UntypedMessage) {
        dispatcher::dispatch_message(dispatch_value, message, self);
    }

    fn handle_on_group_enabled_event(&mut self, _: u8) {}

    fn handle_on_group_disabled_event(&mut self, _: u8) {}
}

impl<const S: usize, const M: usize> MessageHandler for SystemPlugin<S, M> {
    fn handle_stream_keep_alive(&mut self, _: StreamKeepAlive) {}
    fn handle_stream_cancel(&mut self, _: StreamCancel) {}

    fn handle_log(&mut self, message: Log) {
        let stream = self
            .logs
            .entry(message.stream_id())
            .or_insert(StreamReader::new(message.stream_id(), LogReader::default()));

        // конвертируем наш тип в тип creamy_sdk. они одинаковые
        let message = unsafe { core::mem::transmute::<Log, creamy_sdk::logging::Log>(message) };

        let result = match stream.read(message) {
            Ok(value) => value,
            Err(e) => panic!("{e}"),
        };

        if !result {
            return;
        }

        let stream = self.logs.remove(&message.stream_id()).unwrap();
        let reader = stream.into_reader();
        let log_type = reader.log_type();
        let content = reader.into_string().unwrap();

        let target = self.names[message.src as usize - Self::ARRAY_OFFSET]
            .as_ref()
            .unwrap();

        match log_type {
            LogType::Debug => log::debug!(target: target, "{content}"),
            LogType::Info => log::info!(target: target, "{content}"),
            LogType::Warning => log::warn!(target:target, "{content}"),
            LogType::Error => log::error!(target: target, "{content}"),
        }
    }

    fn handle_unknown_message(&mut self, _: u32, _: UntypedMessage) {}
}
