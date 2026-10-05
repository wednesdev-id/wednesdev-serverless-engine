use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
pub use wednes_capabilities::Capabilities;

pub const SUPPORTED_ABI: &str = "wednes:function@0.1.0";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub name: String,
    pub abi: String,
    pub artifact: String,
    #[serde(default)]
    pub runtime: Option<RuntimeConfig>,
    #[serde(default)]
    pub capabilities: Option<Capabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfig {
    pub memory_mb: Option<u32>,
    pub timeout_ms: Option<u64>,
    pub max_concurrency: Option<u32>,
}

impl Manifest {
    pub fn validate(&self) -> Result<()> {
        if self.abi != SUPPORTED_ABI {
            bail!(
                "unsupported ABI: {}, supported: {}",
                self.abi,
                SUPPORTED_ABI
            );
        }

        if self.name.is_empty()
            || self.name.contains('/')
            || self.name.contains('\\')
            || self.name.contains("..")
            || !self
                .name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            bail!("unsafe or invalid manifest name: {}", self.name);
        }

        if self.artifact.is_empty()
            || self.artifact.contains('/')
            || self.artifact.contains('\\')
            || self.artifact.contains("..")
        {
            bail!("unsafe or invalid artifact path: {}", self.artifact);
        }

        if let Some(rt) = &self.runtime {
            if let Some(mem) = rt.memory_mb {
                if !(1..=2048).contains(&mem) {
                    bail!("invalid memory_mb: {}, must be between 1 and 2048 MB", mem);
                }
            }
            if let Some(to) = rt.timeout_ms {
                if !(1..=60_000).contains(&to) {
                    bail!("invalid timeout_ms: {}, must be between 1 and 60000 ms", to);
                }
            }
            if let Some(conc) = rt.max_concurrency {
                if conc == 0 {
                    bail!("invalid max_concurrency: 0, must be >= 1");
                }
            }
        }

        wednes_capabilities::check_capabilities(&self.capabilities)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        Manifest {
            name: "hello-world".into(),
            abi: SUPPORTED_ABI.into(),
            artifact: "hello.wasm".into(),
            runtime: None,
            capabilities: None,
        }
    }

    #[test]
    fn valid_manifest_passes() {
        let m = manifest();
        assert!(m.validate().is_ok());
    }

    #[test]
    fn rejects_unsupported_abi_and_unsafe_name() {
        let mut m = manifest();
        m.abi = "wednes:function@2".into();
        assert!(m.validate().is_err());

        m.abi = SUPPORTED_ABI.into();
        m.name = "../escape".into();
        assert!(m.validate().is_err());

        m.name = "invalid name with space".into();
        assert!(m.validate().is_err());
    }

    #[test]
    fn rejects_unsupported_limits_and_capabilities() {
        let mut m = manifest();
        m.runtime = Some(RuntimeConfig {
            memory_mb: Some(0),
            timeout_ms: Some(0),
            max_concurrency: Some(0),
        });
        assert!(m.validate().is_err());

        // Valid max_concurrency passes without invalid capabilities
        m.runtime = Some(RuntimeConfig {
            memory_mb: Some(32),
            timeout_ms: Some(100),
            max_concurrency: Some(4),
        });
        assert!(m.validate().is_ok());

        m.runtime = Some(RuntimeConfig {
            memory_mb: Some(32),
            timeout_ms: Some(100),
            max_concurrency: Some(1),
        });
        m.capabilities = Some(Capabilities {
            logging: Some(true),
            clock: Some(true),
            environment: None,
            outbound_http: Some(true),
            filesystem: None,
        });
        assert!(m.validate().is_err());
    }

    #[test]
    fn rejects_unknown_fields() {
        let raw = r#"{"name":"hello","abi":"wednes:function@0.1.0","artifact":"hello.wasm","surprise":true}"#;
        assert!(serde_json::from_str::<Manifest>(raw).is_err());
    }
}
