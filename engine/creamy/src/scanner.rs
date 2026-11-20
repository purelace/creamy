use alloc::{string::String, vec::Vec};
use core::fmt::Write;

use creamy_binary::BinaryReader;
use creamy_engine_core::{
    manifest::Package,
    scanner::{Access, ApiScanner},
};
use creamy_sdk::ProtocolId;

fn scan_package(pointer: &[u8]) -> Option<Package> {
    let mut length_bytes = [0u8; 4];
    length_bytes[0] = pointer[0];
    length_bytes[1] = pointer[1];
    length_bytes[2] = pointer[2];
    length_bytes[3] = pointer[3];
    let length = u32::from_le_bytes(length_bytes) as usize;
    let mut data = &pointer[4..length + 4];
    Package::read_ne(&mut data).ok()
}

#[derive(Hash, PartialEq, Eq)]
pub struct CheckedProtocolApi {
    pub access: Access,
    pub name: String,
    pub stable: Option<[u8; 32]>,
    pub extension: Option<[u8; 32]>,
    pub experimental: Option<[u8; 32]>,
}

pub struct PluginApi {
    pub provide: bool,
    pub id: ProtocolId,
    pub protocol: CheckedProtocolApi,
}

fn scan_apis(scanner: &mut impl ApiScanner) -> Vec<PluginApi> {
    // __CREAMY_API_NAME255 = 20
    // __CREAMY_API_CONTRACT255 = 24
    // __CREAMY_API_STABLE_HASH255 = 27
    // __CREAMY_API_EXTENSION_HASH255 = 30
    // __CREAMY_API_EXPERIMENTAL_HASH255 = 33

    let mut inner = Vec::with_capacity(128);

    let mut count = 2; // Skip 0(reserved) and 1(system)

    let mut name = heapless::String::<20>::new();
    let mut contract = heapless::String::<24>::new();
    let mut stable = heapless::String::<27>::new();
    let mut extension = heapless::String::<30>::new();
    let mut experimental = heapless::String::<33>::new();
    loop {
        name.clear();
        contract.clear();
        stable.clear();
        extension.clear();
        experimental.clear();

        let _ = write!(&mut name, "__CREAMY_API_NAME{count}");
        let _ = write!(&mut contract, "__CREAMY_API_CONTRACT{count}");
        let _ = write!(&mut stable, "__CREAMY_API_STABLE_HASH{count}");
        let _ = write!(&mut extension, "__CREAMY_API_EXTENSION_HASH{count}");
        let _ = write!(&mut experimental, "__CREAMY_API_EXPERIMENTAL_HASH{count}");

        let api = scanner.scan_api(&name, &contract, &stable, &extension, &experimental);
        let Some(name) = api.name else {
            break;
        };

        let Some((access, provide)) = api.contract else {
            break;
        };

        inner.push(PluginApi {
            provide,
            id:
            // SAFETY: `count` starts at 2 and increments up to 255, so it is never zero.
            unsafe { ProtocolId::new_unchecked(count) },
            protocol: CheckedProtocolApi {
                access,
                name,
                stable: api.stable,
                extension: api.extension,
                experimental: api.experimental,
            }
        });

        if count == u8::MAX {
            break;
        }
        count += 1;
    }

    inner
}

pub struct Api {
    pub manifest: Option<Package>,
    pub list: Vec<PluginApi>,
}

impl Api {
    pub fn scan(scanner: &mut impl ApiScanner) -> Self {
        Self {
            manifest: scanner
                .scan_manifest("__CREAMY_API_MANIFEST")
                .and_then(scan_package),
            list: scan_apis(scanner),
        }
    }
}
