use std::collections::HashMap;

use creamy_binary::BinaryWriter;
use creamy_manifest::Manifest;
use creamy_xmlc::SemanticModel;

pub struct LinkSectionEntry {
    pub symbol_name: String,
    pub api_name: &'static str,
    pub hash: [u8; 32],
}

pub struct ContractData {
    pub symbol_name: String,
    pub bytes: [u8; 2],
}

pub struct NameEntry {
    pub symbol_name: String,
    pub bytes: Box<[u8]>,
}

pub struct ProtocolEntry {
    pub name: NameEntry,
    pub contract: ContractData,
    pub link_section: [LinkSectionEntry; 3],
}

pub struct ManifestData {
    pub symbol_name: &'static str,
    pub bytes: Box<[u8]>,
}

impl ManifestData {
    fn new(manifest: &Manifest) -> Self {
        let mut buffer = Vec::new();
        manifest.package().write_ne(&mut buffer).unwrap();
        let length = (buffer.len() as u32).to_le_bytes();
        buffer.insert(0, length[3]);
        buffer.insert(0, length[2]);
        buffer.insert(0, length[1]);
        buffer.insert(0, length[0]);

        Self {
            symbol_name: "__CREAMY_API_MANIFEST",
            bytes: buffer.into_boxed_slice(),
        }
    }
}

pub struct ProtocolData {
    pub link_section: &'static str,
    pub entries: Vec<ProtocolEntry>,
}

impl ProtocolData {
    fn new(manifest: &Manifest, models: &HashMap<String, SemanticModel>) -> Self {
        let entries = manifest
            .requested_protocols()
            .iter()
            .map(|(k, v)| (k.clone(), models.get(k).unwrap(), u8::from(v.provide())))
            .enumerate()
            .map(|(index, (api_name, model, provide))| {
                let api = model.extract_api_hashes();
                let access = model.access().as_byte();
                let index = index + 2;
                ProtocolEntry {
                    name: NameEntry {
                        symbol_name: format!("__CREAMY_API_NAME{index}"),
                        bytes: {
                            let mut vec = vec![];
                            vec.extend_from_slice(&(api_name.len() as u32).to_le_bytes());
                            vec.extend_from_slice(api_name.as_bytes());
                            vec.into_boxed_slice()
                        },
                    },
                    contract: ContractData {
                        symbol_name: format!("__CREAMY_API_CONTRACT{index}"),
                        bytes: [access, provide],
                    },
                    link_section: [
                        LinkSectionEntry {
                            symbol_name: format!("__CREAMY_API_STABLE_HASH{index}"),
                            api_name: "stable",
                            hash: api.stable,
                        },
                        LinkSectionEntry {
                            symbol_name: format!("__CREAMY_API_EXTENSION_HASH{index}"),
                            api_name: "extension",
                            hash: api.extension,
                        },
                        LinkSectionEntry {
                            symbol_name: format!("__CREAMY_API_EXPERIMENTAL_HASH{index}"),
                            api_name: "experimental",
                            hash: api.experimental,
                        },
                    ],
                }
            })
            .collect::<Vec<_>>();
        Self {
            entries,
            link_section: ".creamy_api",
        }
    }
}

pub struct LinkSections {
    pub manifest: ManifestData,
    pub protocol: ProtocolData,
}

impl LinkSections {
    pub(crate) fn new(manifest: &Manifest, models: &HashMap<String, SemanticModel>) -> Self {
        Self {
            manifest: ManifestData::new(manifest),
            protocol: ProtocolData::new(manifest, models),
        }
    }
}
