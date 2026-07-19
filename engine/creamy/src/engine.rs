use std::{collections::HashMap, num::NonZeroU8};

use cbus::{MessageBus, config::ValidConfig};
use creamy_cbus_driver::CreamyDriver;
use creamy_engine_core::{
    Constants, PluginLoader, WasmRuntime,
    devkit::{
        BinaryPlugin,
        manifest::{Manifest, RequestedProtocol},
        utils::{strpool::StringPool, version::Version},
        xmlc::{self, ProtocolDefinition, StringPoolResolver},
    },
};

use crate::registry::ProtocolRegistry;

/*
* создаем карту, где название из манифеста сопоставляем с реальной моделью и мета информацией из манифеста.
* если модель не найдена, возвращаем ошибку.

* берем модели из карты и проверяем какие надо задекларировать
* если модель нужно задекларировать, то проверяем, есть ли уже поставщики,
* если есть то ошибка, иначе добавляем.
*
* после этого сверяем запрашиваемые модели.
* если модели различаются, выкидываем ошибку.
*
*/

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Protocol model `{target_model}@{version}` not found")]
    ProtocolModelNotFound {
        target_model: Box<str>,
        version: Version,
    },
    #[error("Protocol `{target_model}@{version}` has already been declared")]
    ProtocolDeclaredAlready {
        target_model: Box<str>,
        version: xmlc::Version,
    },
    #[error(
        "Protocols `{target_model}@{version_a}` and `{target_model}@{version_b}` have different models"
    )]
    DifferentProtocolModels {
        target_model: Box<str>,
        version_a: xmlc::Version,
        version_b: xmlc::Version,
    },
}

struct TempPluginPackage {
    version: Version,
    manifest: Manifest,
    pool: StringPool,
    definitions: HashMap<Box<str>, (ProtocolDefinition, RequestedProtocol)>,
}

impl TempPluginPackage {
    fn from_package(
        BinaryPlugin {
            version,
            manifest,
            pool,
            mut definitions,
            ..
        }: BinaryPlugin,
    ) -> Result<Self, Error> {
        let mut map = HashMap::new();
        for (name, request) in manifest.requested_groups() {
            if let Some(index) = definitions.iter().enumerate().find_map(|(idx, def)| {
                if def.name().resolve(&pool) == name.as_str() {
                    Some(idx)
                } else {
                    None
                }
            }) {
                let definition = definitions.swap_remove(index);
                map.insert(name.as_str().into(), (definition, request.clone()));
            } else {
                return Err(Error::ProtocolModelNotFound {
                    target_model: manifest.name().into(),
                    version,
                });
            }
            //for group_name in request.groups() {
            //}
        }

        Ok(Self {
            version,
            manifest,
            pool,
            definitions: map,
        })
    }
}

pub struct PluginEngine<R: WasmRuntime, L: PluginLoader> {
    bus: MessageBus<CreamyDriver, R::Module>,
    constants: ValidConfig<Constants>,
    runtime: R,
    loader: L,
    registry: ProtocolRegistry,
}

impl<R: WasmRuntime, L: PluginLoader> PluginEngine<R, L> {
    pub fn new(constants: ValidConfig<Constants>, runtime: R, loader: L) -> Self {
        Self {
            bus: MessageBus::new(&constants, CreamyDriver::new),
            constants,
            runtime,
            loader,
            registry: ProtocolRegistry::default(),
        }
    }

    fn init_package(&mut self, package: BinaryPlugin) -> Result<(), Error> {
        let module = self
            .runtime
            .init_module(&self.constants, package.core())
            .unwrap();
        self.bus.add_subscriber(|_, _| module).unwrap();

        let path = format!("{}@{}", package.manifest().name(), package.version());

        let package = TempPluginPackage::from_package(package)?;
        for (name, (mut def, request)) in package.definitions {
            self.registry.replace_strings(&package.pool, &mut def);
            if request.provide() {
                match self.registry.get_model(&name) {
                    Some(def) => {
                        return Err(Error::ProtocolDeclaredAlready {
                            target_model: name,
                            version: def.version(),
                        });
                    }
                    None => {
                        self.registry.declare_protocol(path.clone(), def);
                    }
                }
            } else {
                match self.registry.get_model(&name) {
                    Some(decl) => {
                        if decl != &def {
                            return Err(Error::DifferentProtocolModels {
                                target_model: name,
                                version_a: def.version(),
                                version_b: decl.version(),
                            });
                        }
                        //TODO: provided
                    }
                    None => {
                        //self.registry
                        //    .register_model(format!("{name}@{}", def.version()), def);
                    }
                }
                //self.registry.compare_models(path, definition)
            }
        }

        Ok(())

        //let pool = package.pool;
        //for def in package.definitions.iter_mut() {
        //assert!(self.registry.compare_models(&path, def));
        //}

        //let driver = self.bus.get_driver_mut();
    }

    pub fn tick(&mut self, roundtrip: NonZeroU8) {
        while self.loader.loaded() != 0
            && let Some(package) = self.loader.take_loaded_package()
        {
            self.init_package(package);
        }

        for _ in 0..roundtrip.get() {
            self.bus.tick();
        }
    }

    #[must_use]
    pub const fn loaded_plugins(&self) -> u8 {
        self.bus.subscribers()
    }
}
