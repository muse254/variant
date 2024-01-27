use std::{fs::OpenOptions, path::PathBuf};

use crate::errors::VariantError;

use super::{Metadata, Persist};

/// Reads and writes to the local cache file to provide persistent storage.
pub struct VariantConfig {
    write_path: PathBuf,
}

const VARIANT_FILE: &str = ".variant";

impl VariantConfig {
    pub fn init() -> Result<Self, VariantError> {
        let variant_file = home::home_dir()
            .ok_or_else(|| VariantError::IO("cannot find home directory".into()))?
            .join(VARIANT_FILE);

        // making sure the file exists, if not create it
        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&variant_file)?;

        Ok(Self {
            write_path: variant_file,
        })
    }
}

impl Persist for VariantConfig {
    fn write(&self, metadata: Metadata) -> Result<(), VariantError> {
        let write_ = |to_file| -> Result<(), VariantError> {
            serde_json::to_writer(
                OpenOptions::new().write(true).open(&self.write_path)?,
                to_file,
            )
            .map_err(|e| e.into())
        };

        let mut data = self.read_all()?;
        for m in &mut data {
            if m.username == metadata.username {
                m.username = metadata.username;
                m.email = metadata.email;

                write_(&data)?;
                return Ok(());
            }
        }

        data.push(metadata);
        write_(&data)
    }

    fn read(&self, username: String) -> Result<Option<Metadata>, VariantError> {
        Ok(self
            .read_all()?
            .into_iter()
            .find(|m| m.username == username))
    }

    fn read_all(&self) -> Result<Vec<Metadata>, VariantError> {
        match serde_json::from_str::<Vec<Metadata>>(&std::fs::read_to_string(&self.write_path)?) {
            Ok(data) => Ok(data),
            Err(e) => {
                if e.is_eof() {
                    return Ok(Vec::new());
                }
                Err(e.into())
            }
        }
    }
}
