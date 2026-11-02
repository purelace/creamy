#![allow(clippy::type_complexity)]

mod general;

use criterion::{criterion_group, criterion_main};

mod single {
    use cbus::define_bus;
    use cbus_core::UntypedMessage;
    use criterion::{BatchSize, Criterion};

    use crate::general::init_bus_custom;

    define_bus! {
        max_messages: 1024,
        max_groups: 64,
        max_subscribers: 32,
    }

    fn send_single(_indices: &[u8], out: &mut OutBuf) {
        assert!(out.send_many_iter_exact(std::iter::once(UntypedMessage {
            dst: 2,
            group: 1,
            src: 0,
            kind: 0,
            payload: [0; 28],
        })));
    }

    pub fn send_single_message(c: &mut Criterion) {
        c.bench_function("send_single_message", |b| {
            b.iter_batched_ref(
                || {
                    init_bus_custom::<_, MAX_MESSAGES, MAX_GROUPS, MAX_SUBSCRIBERS>(
                        vec![2],
                        send_single,
                        1,
                    )
                    .unwrap()
                },
                |bus| {
                    bus.tick();
                    bus.tick();
                },
                BatchSize::SmallInput,
            );
        });
    }
}

mod multiple {
    use cbus::define_bus;
    use cbus_core::UntypedMessage;
    use criterion::{BatchSize, Criterion};

    use crate::general::init_bus_custom;

    define_bus! {
        max_messages: 1024,
        max_groups: 64,
        max_subscribers: 32,
    }

    fn send_multiple(indices: &[u8], out: &mut OutBuf) {
        let total_count: usize = indices.iter().map(|&d| d as usize).sum();
        let iter = indices.iter().flat_map(|&dst| {
            let msg = UntypedMessage {
                dst,
                src: 0,
                group: 1,
                kind: 0,
                payload: [0; 28],
            };
            std::iter::repeat_n(msg, dst as usize)
        });

        assert!(
            out.send_many_iter_with_count(iter, total_count),
            "Available capacity: {MAX_MESSAGES}",
        );
    }

    pub fn send_multiple_ordered(c: &mut Criterion) {
        c.bench_function("send_multiple_ordered", |b| {
            b.iter_batched_ref(
                || {
                    init_bus_custom::<_, MAX_MESSAGES, MAX_GROUPS, MAX_SUBSCRIBERS>(
                        vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
                        send_multiple,
                        16,
                    )
                    .unwrap()
                },
                |bus| {
                    bus.tick();
                    bus.tick();
                },
                BatchSize::SmallInput,
            );
        });
    }
}

mod _4k {
    use cbus::define_bus;
    use cbus_core::UntypedMessage;
    use criterion::{BatchSize, Criterion};

    use crate::general::init_bus_custom;

    define_bus! {
        max_messages: 16000,
        max_groups: 128,
        max_subscribers: 32,
    }

    fn send_single_4k_sender(_indices: &[u8], out: &mut OutBuf) {
        let iter = std::iter::repeat_n(
            UntypedMessage {
                dst: 2,
                src: 0,
                group: 1,
                kind: 0,
                payload: [0; 28],
            },
            4096,
        );

        assert!(out.send_many_iter_exact(iter));
    }

    pub fn send_single_4k(c: &mut Criterion) {
        c.bench_function("send_single_4k", |b| {
            b.iter_batched_ref(
                || {
                    init_bus_custom::<_, MAX_MESSAGES, MAX_GROUPS, MAX_SUBSCRIBERS>(
                        vec![2],
                        send_single_4k_sender,
                        1,
                    )
                    .unwrap()
                },
                |bus| {
                    bus.tick();
                    bus.tick();
                },
                BatchSize::LargeInput,
            );
        });
    }
}

mod _65k {
    use cbus::define_bus;
    use cbus_core::UntypedMessage;
    use criterion::{BatchSize, Criterion};

    use crate::general::init_bus_custom;

    define_bus! {
        max_messages: 65535,
        max_groups: 128,
        max_subscribers: 32,
    }

    fn send_mutlitple_65k_sender(indices: &[u8], out: &mut OutBuf) {
        let total_count = indices.len() * 4095;
        let iter = indices.iter().flat_map(|&dst| {
            let msg = UntypedMessage {
                dst,
                src: 0,
                group: 1,
                kind: 0,
                payload: [0; 28],
            };
            std::iter::repeat_n(msg, 4095)
        });

        assert!(out.send_many_iter_with_count(iter, total_count));
    }

    pub fn send_multiple_65k(c: &mut Criterion) {
        c.bench_function("send_multiple_65k", |b| {
            b.iter_batched_ref(
                || {
                    init_bus_custom::<_, MAX_MESSAGES, MAX_GROUPS, MAX_SUBSCRIBERS>(
                        vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
                        send_mutlitple_65k_sender,
                        16,
                    )
                    .unwrap()
                },
                |bus| {
                    bus.tick();
                    bus.tick();
                },
                BatchSize::SmallInput,
            );
        });
    }
}

criterion_group!(
    benches,
    single::send_single_message,
    multiple::send_multiple_ordered,
    _4k::send_single_4k,
    _65k::send_multiple_65k,
);
criterion_main!(benches);
