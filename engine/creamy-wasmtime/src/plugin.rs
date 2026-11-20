use core::ptr::NonNull;

use creamy_engine_core::{
    Module,
    bus::core::Subscriber,
    scanner::{Access, ApiScanner, ProtocolApi},
};
use wasmtime::{Instance, Store, TypedFunc};

pub struct WasmPlugin {
    pub(crate) incoming_ptr: NonNull<u8>,
    pub(crate) outgoing_ptr: NonNull<u8>,
    pub(crate) notify: TypedFunc<(), ()>,
    pub(crate) store: Store<()>,
    pub(crate) instance: Instance,
}

impl ApiScanner for WasmPlugin {
    fn scan_manifest(&mut self, manifest_symbol: &str) -> Option<&[u8]> {
        let offset = self
            .instance
            .get_global(&mut self.store, manifest_symbol)?
            .get(&mut self.store)
            .i32()?;

        let memory = self.instance.get_memory(&mut self.store, "memory")?;
        let memory_data = memory.data(&self.store);
        Some(&memory_data[offset.cast_unsigned() as usize..])
    }

    fn scan_api(
        &mut self,
        name_symbol: &str,
        contract_symbol: &str,
        stable_symbol: &str,
        extension_symbol: &str,
        experimental_symbol: &str,
    ) -> ProtocolApi {
        let name = self
            .instance
            .get_global(&mut self.store, name_symbol)
            .and_then(|g| g.get(&mut self.store).i32())
            .and_then(|o| {
                let memory = self.instance.get_memory(&mut self.store, "memory")?;
                let memory_data = memory.data(&self.store);
                let o = o.cast_unsigned() as usize;
                let length: [u8; 4] = memory_data[o..o + 4]
                    .try_into()
                    .expect("slice with incorrect length");
                let length = u32::from_le_bytes(length);
                let name = &memory_data[o + 4..o + 4 + length as usize];
                let string = String::from_utf8_lossy(name).to_string();
                Some(string)
            });

        let contract = self
            .instance
            .get_global(&mut self.store, contract_symbol)
            .and_then(|g| g.get(&mut self.store).i32())
            .and_then(|o| {
                let memory = self.instance.get_memory(&mut self.store, "memory")?;
                let memory_data = memory.data(&self.store);
                let o = o.cast_unsigned() as usize;
                let access = Access::from_byte(memory_data[o])?;
                let provide = memory_data[o + 1] == 1;
                Some((access, provide))
            });

        let stable = self
            .instance
            .get_global(&mut self.store, stable_symbol)
            .and_then(|g| g.get(&mut self.store).i32())
            .and_then(|o| {
                let memory = self.instance.get_memory(&mut self.store, "memory")?;
                let memory_data = memory.data(&self.store);
                let o = o.cast_unsigned() as usize;
                let hash: [u8; 32] = memory_data[o..o + 32]
                    .try_into()
                    .expect("slice with incorrect length");
                Some(hash)
            });

        let extension = self
            .instance
            .get_global(&mut self.store, extension_symbol)
            .and_then(|g| g.get(&mut self.store).i32())
            .and_then(|o| {
                let memory = self.instance.get_memory(&mut self.store, "memory")?;
                let memory_data = memory.data(&self.store);
                let o = o.cast_unsigned() as usize;
                let hash: [u8; 32] = memory_data[o..o + 32]
                    .try_into()
                    .expect("slice with incorrect length");
                Some(hash)
            });

        let experimental = self
            .instance
            .get_global(&mut self.store, experimental_symbol)
            .and_then(|g| g.get(&mut self.store).i32())
            .and_then(|o| {
                let memory = self.instance.get_memory(&mut self.store, "memory")?;
                let memory_data = memory.data(&self.store);
                let o = o.cast_unsigned() as usize;
                let hash: [u8; 32] = memory_data[o..o + 32]
                    .try_into()
                    .expect("slice with incorrect length");
                Some(hash)
            });

        ProtocolApi {
            name,
            contract,
            stable,
            extension,
            experimental,
        }
    }
}

impl<const M: usize> Module<M> for WasmPlugin {
    fn incoming_ptr(&self) -> NonNull<u8> {
        self.incoming_ptr
    }

    fn outgoing_ptr(&self) -> NonNull<u8> {
        self.outgoing_ptr
    }

    /*
        fn get_protocol_list(&mut self) -> Api {
            Api::scan(
                |symbol| {
                },
                |access, stable, extension, experimental| {

                },
            )
        }
    */
    /*
    #[allow(clippy::cast_sign_loss)]
    fn get_protocol_table(&mut self) -> ProtocolTable<'_> {
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

        ProtocolTable::new(table, table_size, special_table, special_size)
    }*/
}

impl Subscriber for WasmPlugin {
    fn notify(&mut self) {
        self.notify.call(&mut self.store, ()).unwrap();
    }
}
