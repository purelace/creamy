use core::ops::RangeInclusive;
use std::env::consts::ARCH;

use as_guard::AsGuard;

use crate::{
    core::buffer::{Read, Write},
    cpu::{
        arch::AvailableStrategy,
        offsets::{MAX_SLICE_SIZE, Offsets},
    },
    lookup::LookupTable,
    sys::{BufferPool, MessagePool},
};

mod offsets;
mod runner;
mod set;

struct PipelinePlan {
    offsets: Offsets,
    indices: [u8; 256],
}

impl PipelinePlan {
    pub fn new() -> Self {
        Self {
            offsets: Offsets::default(),
            indices: [0; 256],
        }
    }
}

pub trait Strategy {
    fn name() -> &'static str;
    fn features() -> &'static str;
    fn add_offset(offsets: &mut Offsets, count: usize);
    fn get_bucket_idx(count: usize) -> usize;
    fn get_write_ptr(offsets: &Offsets) -> [u8; MAX_SLICE_SIZE];
}

macro_rules! define_strategy {
    {
        $name: ident,
        $last_from:literal .. => [$last_field: ident | $last_runner:ident] $(,)?
        $(
            $from:literal .. $to:literal => [$field: ident | $runner:ident] $(,)?
        )*
    } => {
        use $crate::cpu::{
            Strategy,
            StrategyRunner,
            MessagePipeline,
            PipelineData,
            offsets::{Offsets, MAX_SLICE_SIZE},
            set::InstructionSet,
            runner::InstructionRunner
        };

        pub struct $name;
        impl Strategy for $name {
            fn name() -> &'static str {
                stringify!($name)
            }

            fn features() -> &'static str {
                ""
            }

            #[inline(always)]
            fn add_offset(offsets: &mut Offsets, count: usize) {
                offsets.$last_field += u8::from(count >= $last_from);
                $(
                    offsets.$field += u8::from(($from..$to).contains(&count));
                )*
                offsets.ignore = u8::from(count == 0);
            }

            #[inline(always)]
            fn get_bucket_idx(count: usize) -> usize {
                0 $( + usize::from(count < $to))* + usize::from(count == 0)
            }

            #[inline(always)]
            fn get_write_ptr(offsets: &Offsets) -> [u8; MAX_SLICE_SIZE] {
                let mut ptrs = [0u8; MAX_SLICE_SIZE];
                let mut current = 0;

                current += offsets.$last_field;

                let mut i = 1;
                $(
                    ptrs[i] = current;
                    current += offsets.$field;
                    i += 1;
                )*

                ptrs[i] = current;

                ptrs
            }
        }

        impl StrategyRunner for $name {
            #[inline(always)]
            fn run(pipeline: &mut MessagePipeline, data: &mut PipelineData) {
                let mut total_offset = 0;

                macro_rules! slice {
                    ($inner_field: ident) => {{
                        let offset = pipeline.plan.offsets.$inner_field as usize;
                        let indices_slice = &pipeline.plan.indices[total_offset..total_offset + offset];
                        let indices_slice = unsafe {&*std::ptr::from_ref(indices_slice)};
                        #[allow(unused_assignments)]
                        {
                            total_offset += offset;
                        }
                        indices_slice
                    }};
                }

                if pipeline.plan.offsets.$last_field != 0 {
                    $last_runner::prepare_batches(slice!($last_field), data);
                    pipeline.batch_messages(data);
                    $last_runner::send_batches(pipeline, data);
                    pipeline.batch.clear();
                    data.memory.message.clear();
                }

                $(
                    if pipeline.plan.offsets.$field != 0 {
                        $runner::prepare_and_send_direct_all(slice!($field), data);
                    }
                )*
            }
        }
    };
}

mod arch;

impl PipelinePlan {
    #[inline(always)]
    fn prepare<S: Strategy>(&mut self, data: &PipelineData) {
        self.offsets.reset();
        self.indices = [0; 256];

        //TODO fix
        //let count = data.memory.read.u_count_for(0);
        //S::add_offset(&mut self.offsets, count);
        for src in data.subscriber_range.clone() {
            let count = data.memory.read.u_count_for(src);
            S::add_offset(&mut self.offsets, count);
        }

        // 2. Получаем правильные начальные позиции (0, len_batch, len_batch + len_avx...)
        let mut write_ptr = S::get_write_ptr(&self.offsets);

        // 3. Заполняем массив indices, используя смещения
        for src in data.subscriber_range.clone() {
            let count = data.memory.read.u_count_for(src);
            let bucket_idx = S::get_bucket_idx(count);

            let pos = write_ptr[bucket_idx];
            self.indices[pos as usize] = src as u8;
            write_ptr[bucket_idx] += 1; // Сдвигаемся внутри бакета
        }
    }
}

pub trait StrategyRunner {
    fn run(pipeline: &mut MessagePipeline, data: &mut PipelineData);
}

pub struct MessagePipeline {
    plan: PipelinePlan,
    // dst, count, ptr_location
    batch: Vec<(u8, u32, usize)>,
}

impl MessagePipeline {
    pub fn new(max_subscribers: u8) -> Self {
        //println!("[MessageBus] Arch: {ARCH}");
        //println!("[MessageBus] Stragegy: {}", AvailableStrategy::name());
        //println!(
        //    "[MessageBus] Features in use: {}",
        //    AvailableStrategy::features()
        //);

        Self {
            plan: PipelinePlan::new(),
            batch: Vec::with_capacity(max_subscribers as usize),
        }
    }

    #[inline(never)]
    pub(crate) fn dispatch_messages(&mut self, data: &mut PipelineData) {
        self.plan.prepare::<AvailableStrategy>(data);
        AvailableStrategy::run(self, data);
    }

    #[inline(always)]
    fn sort_messages(data: &mut PipelineData) {
        data.memory
            .message
            .as_mut_slice()
            .sort_unstable_by_key(|m| m.dst);
    }

    #[inline(always)]
    pub(crate) fn batch_messages(&mut self, data: &mut PipelineData) {
        Self::sort_messages(data);
        let mut ptr_location = 0;
        let pool_slice = data.memory.message.as_slice();

        // Находим индекс первого элемента, где dst != 0
        let start_index = pool_slice.partition_point(|x| x.dst == 0);
        let active_slice = &pool_slice[start_index..];

        // Обновляем начальный ptr_location, чтобы он соответствовал пропущенным данным
        ptr_location += start_index;

        active_slice
            .chunk_by(|a, b| a.dst == b.dst)
            .for_each(|slice| {
                let dst = slice[0].dst; // slice гарантированно не пустой
                self.batch.push((dst, slice.len().safe_as(), ptr_location));
                ptr_location += slice.len();
            });
    }
}

pub struct PipelineData<'a> {
    pub(crate) lookup_table: &'a LookupTable,
    pub(crate) memory: MemoryPools<'a>,
    pub(crate) subscriber_range: RangeInclusive<usize>,
}

pub struct MemoryPools<'a> {
    pub(crate) write: &'a mut BufferPool<Read>,
    pub(crate) read: &'a mut BufferPool<Write>,
    pub(crate) message: &'a mut MessagePool,
}
