use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use wednes_gateway::create_router;
use wednes_runtime::RuntimeBackend;
use wednes_runtime_wasmtime::WasmtimeBackend;
use wednes_core::{manifest::Manifest, registry::FileRegistry};
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about = "Wednes Engine Function Daemon")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    // Backward compat fallback
    #[arg(short, long, default_value = "127.0.0.1:8080", global = true)]
    listen: String,
    #[arg(short, long, global = true)]
    wasm: Option<PathBuf>,
    #[arg(short, long, default_value = "hello", global = true)]
    id: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Deploy {
        #[arg(short, long)]
        manifest: PathBuf,
        #[arg(short, long)]
        registry: PathBuf,
    },
    Run {
        #[arg(short, long, default_value = "127.0.0.1:8080")]
        listen: String,
        #[arg(short, long)]
        registry: PathBuf,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        Some(Commands::Deploy { manifest, registry }) => {
            let m_bytes = fs::read(&manifest)?;
            let manifest_str = String::from_utf8(m_bytes)?;
            let m: Manifest = toml::from_str(&manifest_str)?;
            
            // Validate manifest limits etc
            m.validate()?;
            
            // Precompile the component and cache it
            info!("Precompiling artifact: {}", m.artifact);
            let mut backend = WasmtimeBackend::init()?;
            let artifact_path = manifest.parent().unwrap_or(std::path::Path::new("")).join(&m.artifact);
            if !artifact_path.exists() {
                bail!("Artifact not found: {:?}", artifact_path);
            }
            backend.precompile(&m.name, &artifact_path)?;
            
            // Register
            let mut reg = FileRegistry::open(&registry)?;
            reg.register(m.clone())?;
            info!("Deployed function '{}' to registry {:?}", m.name, registry);
            Ok(())
        },
        Some(Commands::Run { listen, registry }) => {
            let reg = FileRegistry::open(&registry)?;
            let mut backend = WasmtimeBackend::init()?;
            for (id, manifest) in reg.data.functions.iter() {
                // Here we rely on Wasmtime's cache since we call load_module with the artifact path
                let artifact_path = registry.parent().unwrap_or(std::path::Path::new("")).join(&manifest.artifact);
                backend.load_manifest(manifest, &artifact_path)?;
                info!("Loaded function '{}'", id);
            }
            let runtime: Arc<dyn RuntimeBackend> = Arc::new(backend);
            let app = create_router(runtime);
            let listener = tokio::net::TcpListener::bind(&listen).await?;
            info!("wednesd HTTP gateway listening on {}", listen);
            axum::serve(listener, app).await?;
            Ok(())
        },
        None => {
            // Backward compat
            info!("Starting wednesd on {} (legacy mode)", args.listen);
            let mut backend = WasmtimeBackend::init()?;
            if let Some(wasm_path) = args.wasm {
                info!("Loading function '{}' from {:?}", args.id, wasm_path);
                backend.load_module(&args.id, &wasm_path)?;
            }
            let runtime: Arc<dyn RuntimeBackend> = Arc::new(backend);
            let app = create_router(runtime);
            let listener = tokio::net::TcpListener::bind(&args.listen).await?;
            info!("wednesd HTTP gateway listening on {}", args.listen);
            axum::serve(listener, app).await?;
            Ok(())
        }
    }
}
