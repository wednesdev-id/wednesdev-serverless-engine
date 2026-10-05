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
    components: HashMap<String, (Component, Option<wednes_core::manifest::Manifest>)>,
}

pub struct HostCtx {
    table: ResourceTable,
    wasi: WasiCtx,
    limits: StoreLimits,
    outbound_http_allowed: bool,
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

#[async_trait]
impl wednes::function::http_client::Host for HostCtx {
    async fn send_request(
        &mut self,
        req: wednes::function::types::Request,
    ) -> Result<wednes::function::types::Response, String> {
        if !self.outbound_http_allowed {
            return Err("permission denied: outbound_http capability is not enabled".to_string());
        }

        let method = match req.method.to_uppercase().as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            "PUT" => reqwest::Method::PUT,
            "DELETE" => reqwest::Method::DELETE,
            "PATCH" => reqwest::Method::PATCH,
            _ => return Err(format!("unsupported method: {}", req.method)),
        };

        let mut builder = reqwest::Client::new().request(method, &req.path);
        for header in req.headers {
            builder = builder.header(header.name, header.value);
        }

        if !req.body.is_empty() {
            builder = builder.body(req.body);
        }

        let res = match builder.send().await {
            Ok(res) => res,
            Err(e) => return Err(e.to_string()),
        };

        let status = res.status().as_u16();

        let mut headers = Vec::new();
        for (name, value) in res.headers().iter() {
            if let Ok(val_str) = value.to_str() {
                headers.push(wednes::function::types::Header {
                    name: name.as_str().to_string(),
                    value: val_str.to_string(),
                });
            }
        }

        let body = match res.bytes().await {
            Ok(b) => b.to_vec(),
            Err(e) => return Err(e.to_string()),
        };

        Ok(wednes::function::types::Response {
            status,
            headers,
            body,
        })
    }
}

#[async_trait]
impl RuntimeBackend for WasmtimeBackend {
    fn init() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.epoch_interruption(true);
        config.cache_config_load_default()?;

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        Function::add_to_linker(&mut linker, |state: &mut HostCtx| state)?;

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
            .insert(manifest.name.clone(), (component, Some(manifest.clone())));
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
        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();
        let limits = StoreLimitsBuilder::new()
            .memory_size(
                runtime
                    .as_ref()
                    .and_then(|r| r.runtime.as_ref().and_then(|rt| rt.memory_mb))
                    .unwrap_or(32) as usize
                    * 1024
                    * 1024,
            )
            .build();

        let outbound_http_allowed = wednes_capabilities::outbound_http_allowed(
            runtime.as_ref().and_then(|r| r.capabilities.as_ref()),
        );

        let ctx = HostCtx {
            table,
            wasi,
            limits,
            outbound_http_allowed,
        };
        let mut store = Store::new(&self.engine, ctx);
        store.limiter(|state| &mut state.limits);

        store.epoch_deadline_async_yield_and_update(1);

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

        let timeout = Duration::from_millis(
            runtime
                .as_ref()
                .and_then(|r| r.runtime.as_ref().and_then(|rt| rt.timeout_ms))
                .unwrap_or(2000),
        );

        let exec = async {
            let bindings = Function::instantiate_async(&mut store, component, &self.linker).await?;
            bindings.call_handle(&mut store, &req).await
        };

        let res = match tokio::time::timeout(timeout, exec).await {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => bail!("Invocation error: {}", e),
            Err(_) => bail!("timeout"),
        };

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
