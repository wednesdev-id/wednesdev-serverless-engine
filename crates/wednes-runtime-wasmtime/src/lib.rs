use anyhow::{bail, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Config, Engine, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wednes_core::{Header as CoreHeader, Request as CoreRequest, Response as CoreResponse};
use wednes_runtime::RuntimeBackend;

bindgen!({
    world: "function",
    path: "../../wit",
    async: true
});

pub struct WasmtimeBackend {
    engine: Engine,
    linker: Linker<HostCtx>,
    components: HashMap<String, (Component, Option<wednes_core::manifest::RuntimeConfig>)>,
}

pub struct HostCtx {
    table: ResourceTable,
    wasi: WasiCtx,
    limits: StoreLimits,
}

impl WasiView for HostCtx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl wednes::function::types::Host for HostCtx {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_backend_releases_engine() {
        let backend = WasmtimeBackend::init().unwrap();
        let weak = backend.engine.weak();
        drop(backend);
        assert!(weak.upgrade().is_none(), "epoch ticker retained engine");
    }
}

#[async_trait]
impl RuntimeBackend for WasmtimeBackend {
    fn init() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.epoch_interruption(true);
        // In-memory compiled cache only; no native artifact deserialization.

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        // Background epoch ticker: 1 tick = 25ms
        let ticker_engine = engine.weak();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(25));
            if let Some(engine) = ticker_engine.upgrade() {
                engine.increment_epoch();
            } else {
                break;
            }
        });

        Ok(Self {
            engine,
            linker,
            components: HashMap::new(),
        })
    }

    fn load_module(&mut self, id: &str, wasm_path: &Path) -> Result<()> {
        let component = Component::from_file(&self.engine, wasm_path)?;
        FunctionPre::new(self.linker.instantiate_pre(&component)?)?;
        self.components.insert(id.to_string(), (component, None));
        Ok(())
    }

    fn load_manifest(
        &mut self,
        manifest: &wednes_core::manifest::Manifest,
        wasm_path: &Path,
    ) -> Result<()> {
        manifest.validate()?;
        let component = Component::from_file(&self.engine, wasm_path)?;
        FunctionPre::new(self.linker.instantiate_pre(&component)?)?;
        self.components
            .insert(manifest.name.clone(), (component, manifest.runtime.clone()));
        Ok(())
    }

    fn precompile(&mut self, id: &str, wasm_path: &Path) -> Result<()> {
        let component = Component::from_file(&self.engine, wasm_path)?;
        FunctionPre::new(self.linker.instantiate_pre(&component)?)?;
        FunctionPre::new(self.linker.instantiate_pre(&component)?)?;
        self.components.insert(id.to_string(), (component, None));
        Ok(())
    }

    fn load_precompiled(&mut self, _id: &str) -> Result<()> {
        bail!("persistent native cache unsupported; load source WASM at startup")
    }

    async fn execute(&self, id: &str, request: CoreRequest) -> Result<CoreResponse> {
        let (component, runtime) = match self.components.get(id) {
            Some(c) => c,
            None => bail!("Function {} not found", id),
        };

        let table = ResourceTable::new();
        let wasi = WasiCtxBuilder::new().inherit_stdio().build();
        let limits = StoreLimitsBuilder::new()
            .memory_size(
                runtime.as_ref().and_then(|r| r.memory_mb).unwrap_or(32) as usize * 1024 * 1024,
            )
            .build();

        let ctx = HostCtx {
            table,
            wasi,
            limits,
        };
        let mut store = Store::new(&self.engine, ctx);
        store.limiter(|state| &mut state.limits);

        store.set_epoch_deadline(
            runtime
                .as_ref()
                .and_then(|r| r.timeout_ms)
                .unwrap_or(2000)
                .div_ceil(25),
        );
        store.epoch_deadline_trap();

        let bindings = Function::instantiate_async(&mut store, component, &self.linker).await?;

        let req = wednes::function::types::Request {
            method: request.method,
            path: request.path,
            headers: request
                .headers
                .into_iter()
                .map(|h| wednes::function::types::Header {
                    name: h.name,
                    value: h.value,
                })
                .collect(),
            body: request.body,
        };

        let res = bindings.call_handle(&mut store, &req).await?;

        Ok(CoreResponse {
            status: res.status,
            headers: res
                .headers
                .into_iter()
                .map(|h| CoreHeader {
                    name: h.name,
                    value: h.value,
                })
                .collect(),
            body: res.body,
        })
    }
}
