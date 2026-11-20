use alloc::string::String;

#[derive(PartialEq, Eq, Hash)]
pub enum Access {
    /// Single provider - many consumers
    Public = 0,

    /// Single provider - single consumer
    Private = 1,
}

impl Access {
    #[must_use]
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Public),
            1 => Some(Self::Private),
            _ => None,
        }
    }
}

pub struct ProtocolApi {
    pub name: Option<String>,
    pub contract: Option<(Access, bool)>,
    pub stable: Option<[u8; 32]>,
    pub extension: Option<[u8; 32]>,
    pub experimental: Option<[u8; 32]>,
}

pub trait ApiScanner {
    fn scan_manifest(&mut self, manifest_symbol: &str) -> Option<&[u8]>;
    fn scan_api(
        &mut self,
        name_symbol: &str,
        contract_symbol: &str,
        stable_symbol: &str,
        extension_symbol: &str,
        experimental_symbol: &str,
    ) -> ProtocolApi;
}

impl ApiScanner for () {
    fn scan_manifest(&mut self, _: &str) -> Option<&[u8]> {
        None
    }

    fn scan_api(&mut self, _: &str, _: &str, _: &str, _: &str, _: &str) -> ProtocolApi {
        ProtocolApi {
            name: None,
            contract: None,
            stable: None,
            extension: None,
            experimental: None,
        }
    }
}
