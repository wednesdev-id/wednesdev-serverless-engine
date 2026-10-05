use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Capabilities {
    pub logging: Option<bool>,
    pub clock: Option<bool>,
    pub environment: Option<bool>,
    pub outbound_http: Option<bool>,
    pub filesystem: Option<bool>,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            logging: Some(true),
            clock: Some(true),
            environment: Some(false),
            outbound_http: Some(false),
            filesystem: Some(false),
        }
    }
}

impl Capabilities {
    pub fn check(&self) -> Result<()> {
        if self.logging.is_some() || self.clock.is_some() {
            bail!("logging/clock capability configuration unsupported until M4");
        }
        if self.filesystem == Some(true) {
            bail!("unsupported capability: filesystem is not supported in M3");
        }
        if self.environment == Some(true) {
            bail!("unsupported capability: environment is not supported in M3");
        }
        Ok(())
    }

    pub fn is_outbound_http_allowed(&self) -> bool {
        self.outbound_http.unwrap_or(false)
    }

    pub fn is_logging_allowed(&self) -> bool {
        self.logging.unwrap_or(true)
    }

    pub fn is_clock_allowed(&self) -> bool {
        self.clock.unwrap_or(true)
    }

    pub fn is_filesystem_allowed(&self) -> bool {
        self.filesystem.unwrap_or(false)
    }

    pub fn is_environment_allowed(&self) -> bool {
        self.environment.unwrap_or(false)
    }
}

pub fn check_capabilities(capabilities: &Option<Capabilities>) -> Result<()> {
    if let Some(caps) = capabilities {
        caps.check()?;
    }
    Ok(())
}

pub fn outbound_http_allowed(capabilities: Option<&Capabilities>) -> bool {
    capabilities
        .map(|c| c.is_outbound_http_allowed())
        .unwrap_or(false)
}

pub fn default_capabilities() -> Capabilities {
    Capabilities::default()
}
