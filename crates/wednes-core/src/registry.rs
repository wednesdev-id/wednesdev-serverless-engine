use crate::manifest::Manifest;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub functions: BTreeMap<String, Manifest>,
}

pub struct FileRegistry {
    path: PathBuf,
    pub data: Registry,
}
impl FileRegistry {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let data: Registry = if path.exists() {
            serde_json::from_slice(&fs::read(&path)?)?
        } else {
            Registry::default()
        };
        for (id, manifest) in &data.functions {
            manifest.validate()?;
            if id != &manifest.name {
                bail!("registry ID mismatch: {id}");
            }
        }
        Ok(Self { path, data })
    }
    pub fn register(&mut self, m: Manifest) -> Result<()> {
        m.validate()?;
        let mut next = self.data.clone();
        next.functions.insert(m.name.clone(), m);
        let tmp = self.path.with_extension("tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(&next)?)?;
        if let Err(e) = fs::rename(&tmp, &self.path).context("atomic registry update") {
            let _ = fs::remove_file(tmp);
            return Err(e);
        }
        self.data = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    #[test]
    fn invalid_deploy_keeps_existing_registry() {
        let p = std::env::temp_dir().join(format!(
            "wednes-reg-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut r = FileRegistry::open(&p).unwrap();
        let m = Manifest {
            name: "hello".into(),
            abi: "wednes:function@0.1.0".into(),
            artifact: "x.wasm".into(),
            runtime: None,
            capabilities: None,
        };
        r.register(m.clone()).unwrap();
        let bad = Manifest {
            name: "../bad".into(),
            ..m
        };
        assert!(r.register(bad).is_err());
        assert_eq!(r.data.functions.len(), 1);
        assert_eq!(FileRegistry::open(&p).unwrap().data.functions.len(), 1);
        let _ = fs::remove_file(p);
    }
}
