use core::marker::PhantomData;

use arrayvec::ArrayVec;
use as_guard::AsGuard;
use cbus_core::{Subscriber, buffer::reference::RefMutBuf};

use crate::{
    bus::SubscriberData,
    cpu::{
        arch::AvailableStrategy,
        offsets::{MAX_SLICE_SIZE, Offsets},
    },
    lookup::LookupTable,
    sys::{MessagePool, SubscriberRange},
};

mod offsets;
mod runner;
mod set;

#[derive(Debug)]
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
    fn add_offset(offsets: &mut Offsets, count: usize);
    fn get_bucket_idx(count: usize) -> usize;
    fn get_write_ptr(offsets: Offsets) -> [u8; MAX_SLICE_SIZE];
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
            fn get_write_ptr(offsets: Offsets) -> [u8; MAX_SLICE_SIZE] {
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

        impl<S, const MM: usize, const MG: usize, const MS: usize> StrategyRunner<S, MM, MG, MS> for $name
        where
            S: $crate::core::Subscriber,
        {
            #[cfg_attr(feature = "tracing",
                tracing::instrument(skip_all, name = "bus::run", level = "trace")
            )]
            #[inline(always)]
            fn run(pipeline: &mut MessagePipeline<S, MM, MG, MS>, data: &mut PipelineData<S, MM, MG, MS>) {
                let mut total_offset = 0;

                macro_rules! slice {
                    ($inner_field: ident) => {{
                        let offset = pipeline.plan.offsets.$inner_field as usize;
                        let indices_slice = &pipeline.plan.indices[total_offset..total_offset + offset];
                        let indices_slice = unsafe {&*core::ptr::from_ref(indices_slice)};
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
    fn prepare<ST, SU, const MM: usize, const MG: usize, const MS: usize>(
        &mut self,
        data: &PipelineData<SU, MM, MG, MS>,
    ) where
        ST: Strategy,
        SU: Subscriber,
    {
        self.offsets.reset();
        self.indices = [0; 256];

        for src in data.subscriber_range {
            let sub_data = &data.memory.subscribers[src.as_usize()];
            let count = sub_data.outgoing_ref().count();
            ST::add_offset(&mut self.offsets, count as usize);
        }

        // 2. Получаем правильные начальные позиции (0, len_batch, len_batch + len_avx...)
        let mut write_ptr = ST::get_write_ptr(self.offsets);

        // 3. Заполняем массив indices, используя смещения
        for src in data.subscriber_range {
            let sub_data = &data.memory.subscribers[src.as_usize()];
            let count = sub_data.outgoing_ref().count();

            let bucket_idx = ST::get_bucket_idx(count as usize);

            let pos = write_ptr[bucket_idx];
            self.indices[pos as usize] = src.as_u8();
            write_ptr[bucket_idx] += 1; // Сдвигаемся внутри бакета
        }
    }
}

pub trait StrategyRunner<S, const MM: usize, const MG: usize, const MS: usize>
where
    S: Subscriber,
{
    fn run(pipeline: &mut MessagePipeline<S, MM, MG, MS>, data: &mut PipelineData<S, MM, MG, MS>);
}

#[derive(Debug)]
pub struct MessagePipeline<S: Subscriber, const MM: usize, const MG: usize, const MS: usize> {
    plan: PipelinePlan,
    // dst, count, ptr_location
    batch: ArrayVec<(u8, u32, usize), MS>,
    _phantom: PhantomData<S>,
}

impl<S, const MM: usize, const MG: usize, const MS: usize> MessagePipeline<S, MM, MG, MS>
where
    S: Subscriber,
{
    pub fn new() -> Self {
        Self {
            plan: PipelinePlan::new(),
            batch: ArrayVec::new(),
            _phantom: PhantomData,
        }
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(skip_all, name = "bus::dispatch_messages", level = "trace")
    )]
    pub(crate) fn dispatch_messages(&mut self, data: &mut PipelineData<S, MM, MG, MS>) {
        self.plan.prepare::<AvailableStrategy, S, MM, MG, MS>(data);
        AvailableStrategy::run(self, data);
    }

    #[inline(always)]
    fn sort_messages(data: &mut PipelineData<S, MM, MG, MS>) {
        data.memory
            .message
            .as_mut_slice()
            .sort_unstable_by_key(|m| m.dst);
    }

    #[inline(always)]
    pub(crate) fn batch_messages(&mut self, data: &mut PipelineData<S, MM, MG, MS>) {
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

pub struct PipelineData<'a, S: Subscriber, const MM: usize, const MG: usize, const MS: usize> {
    pub(crate) lookup_table: &'a LookupTable<MG, MS>,
    pub(crate) memory: MemoryPools<'a, S, MM, MS>,
    pub(crate) subscriber_range: SubscriberRange,
}

pub struct MemoryPools<'a, S: Subscriber, const MM: usize, const MS: usize> {
    pub(crate) subscribers: &'a mut [SubscriberData<S, MM>; MS],
    pub(crate) message: &'a mut MessagePool<MM, MS>,
}

impl<S: Subscriber, const MM: usize, const MS: usize> MemoryPools<'_, S, MM, MS> {
    #[cfg(target_feature = "sse4.1")]
    pub const fn get_inc_raw_buf(&mut self, index: u8) -> cbus_core::buffer::reference::RawBuf {
        let data = &mut self.subscribers[index as usize];
        unsafe { data.incoming_raw_ptr() }
    }

    /// Write
    pub const fn get_mut_inc_buf(&mut self, index: usize) -> RefMutBuf<'_, MM> {
        unsafe {
            let ptr = self.subscribers.as_mut_ptr().add(index);
            (*ptr).incoming_mut()
        }
    }

    /// Read
    pub const fn get_mut_out_buf<'a>(&mut self, index: u8) -> RefMutBuf<'a, MM> {
        unsafe {
            let ptr = self.subscribers.as_mut_ptr().add(index as usize);
            (*ptr).outgoing_mut()
        }
    }
}
