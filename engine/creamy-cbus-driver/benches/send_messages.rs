#![allow(clippy::type_complexity)]

mod general;

use cbus::{
    MessageBus,
    core::{UntypedMessage, buffer::OutBuf},
};
use creamy_cbus_driver::CreamyDriver;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

use self::general::Legacy;
use crate::general::{
    MAX_MESSAGES, SimpleSubscriber, init_bus_legacy, init_bus_legacy_large_buf,
    init_bus_legacy_ularge_buf,
};

fn send_single<const M: usize>(_indices: &[u8], out: &mut OutBuf<M>) {
    assert!(out.send_many_iter_exact(std::iter::once(UntypedMessage {
        dst: 2,
        group: 1,
        src: 0,
        kind: 0,
        payload: [0; 28],
    })));
}

fn send_single_message(c: &mut Criterion) {
    c.bench_function("send_single_message", |b| {
        b.iter_batched_ref(
            || init_bus_legacy(vec![2], send_single, 1).unwrap(),
            |bus| {
                bus.tick();
                bus.tick();
            },
            BatchSize::SmallInput,
        );
    });
}

fn send_multiple<const M: usize>(indices: &[u8], out: &mut OutBuf<M>) {
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
        "Available capacity: {M}",
    );
}

fn send_multiple_ordered(c: &mut Criterion) {
    c.bench_function("send_multiple_ordered", |b| {
        b.iter_batched_ref(
            || {
                init_bus_legacy(
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

fn send_single_4k_sender<const M: usize>(_indices: &[u8], out: &mut OutBuf<M>) {
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

fn send_single_4k(c: &mut Criterion) {
    c.bench_function("send_single_4k", |b| {
        b.iter_batched_ref(
            || init_bus_legacy_large_buf(vec![2], send_single_4k_sender, 1).unwrap(),
            |bus| {
                bus.tick();
                bus.tick();
            },
            BatchSize::LargeInput,
        );
    });
}

fn send_mutlitple_65k_sender<const M: usize>(indices: &[u8], out: &mut OutBuf<M>) {
    let total_count = indices.len() * 4096;
    let iter = indices.iter().flat_map(|&dst| {
        let msg = UntypedMessage {
            dst,
            src: 0,
            group: 1,
            kind: 0,
            payload: [0; 28],
        };
        std::iter::repeat_n(msg, 4096)
    });

    assert!(out.send_many_iter_with_count(iter, total_count));
}

fn send_multiple_65k(c: &mut Criterion) {
    c.bench_function("send_multiple_65k", |b| {
        b.iter_batched_ref(
            || {
                init_bus_legacy_ularge_buf(
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

fn empty(c: &mut Criterion) {
    let mut bus =
        MessageBus::<Legacy, CreamyDriver, MAX_MESSAGES, SimpleSubscriber<MAX_MESSAGES>>::new(
            CreamyDriver::new::<Legacy>(),
        );

    c.bench_function("empty", |b| {
        b.iter(|| {
            bus.tick();
        });
    });
}

criterion_group!(
    benches,
    empty,
    send_single_message,
    send_multiple_ordered,
    send_single_4k,
    send_multiple_65k,
);
criterion_main!(benches);
