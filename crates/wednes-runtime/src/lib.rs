use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use wednes_core::{manifest::Manifest, Request, Response};

#[async_trait]
pub trait RuntimeBackend: Send + Sync {
    /// Initialize the backend (engines, linkers, etc).
    fn init() -> Result<Self>
    where
        Self: Sized;

    /// Compile or load a function module.
    fn load_module(&mut self, id: &str, wasm_path: &Path) -> Result<()>;

    /// Load a function using its manifest configuration.
    fn load_manifest(&mut self, manifest: &Manifest, wasm_path: &Path) -> Result<()>;

    /// Precompile a function module and store in managed cache.
    fn precompile(&mut self, id: &str, wasm_path: &Path) -> Result<()>;

    /// Load a function module from the managed precompiled cache.
    fn load_precompiled(&mut self, id: &str) -> Result<()>;

    /// Execute the loaded module with a given request.
    async fn execute(&self, id: &str, request: Request) -> Result<Response>;
}
