use core::ptr::NonNull;

use creamy_engine_core::{GroupTable, Module, bus::core::Subscriber};
use wasmtime::{Instance, Store, TypedFunc};

pub struct WasmPlugin {
    pub(crate) incoming_ptr: NonNull<u8>,
    pub(crate) outgoing_ptr: NonNull<u8>,
    pub(crate) notify: TypedFunc<(), ()>,
    pub(crate) store: Store<()>,
    pub(crate) instance: Instance,
}

impl<const M: usize> Module<M> for WasmPlugin {
    fn incoming_ptr(&self) -> NonNull<u8> {
        self.incoming_ptr
    }

    fn outgoing_ptr(&self) -> NonNull<u8> {
        self.outgoing_ptr
    }

    #[allow(clippy::cast_sign_loss)]
    fn get_group_table(&mut self) -> GroupTable<'_> {
        let (table_size, table_range) = {
            let size_ptr = self
                .instance
                .get_global(&mut self.store, "__INTERNAL__TABLE_SIZE")
                .unwrap()
                .get(&mut self.store)
                .i32()
                .unwrap();

            let memory = self.instance.get_memory(&mut self.store, "memory").unwrap();
            let memory_data = memory.data(&self.store);
            let size = memory_data[size_ptr as usize];

            let ptr = self
                .instance
                .get_global(&mut self.store, "__INTERNAL__TABLE")
                .unwrap();

            let offset = ptr.get(&mut self.store).i32().unwrap() as usize;
            (size, offset..offset + size as usize)
        };

        let (special_size, special_range) = {
            let size_ptr = self
                .instance
                .get_global(&mut self.store, "__INTERNAL__SPECIAL_TABLE_SIZE")
                .unwrap()
                .get(&mut self.store)
                .i32()
                .unwrap();

            let memory = self.instance.get_memory(&mut self.store, "memory").unwrap();
            let memory_data = memory.data(&self.store);
            let size = memory_data[size_ptr as usize];

            let ptr = self
                .instance
                .get_global(&mut self.store, "__INTERNAL__SPECIAL")
                .unwrap();

            let offset = ptr.get(&mut self.store).i32().unwrap() as usize;
            let expected_bytes_len = size as usize * core::mem::size_of::<u64>();
            (size, offset..offset + expected_bytes_len)
        };

        let memory = self.instance.get_memory(&mut self.store, "memory").unwrap();
        let memory_data = memory.data(&self.store);
        let table = &memory_data[table_range];
        let special_table = unsafe { memory_data[special_range].align_to::<u64>().1 };

        GroupTable::new(table, table_size, special_table, special_size)
    }
}

impl Subscriber for WasmPlugin {
    fn notify(&mut self) {
        self.notify.call(&mut self.store, ()).unwrap();
    }
}
