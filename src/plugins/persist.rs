use std::fs::{File, OpenOptions};

use super::{Metadata, Persist};

// Reads and writes to the local cache file to provide persistent storage.
pub(crate) struct VariantConfig {
    write_path: File,
}

impl VariantConfig {
    pub fn init() -> Result<Self, Vec<u8>> {
        let variant_file = home::home_dir()
            .ok_or_else(|| b"cannot find home directory")?
            .join(".variant");

        Ok(Self {
            write_path: OpenOptions::new()
                .write(true)
                .create(true)
                .open(variant_file)
                .map_err(|e| e.to_string().as_bytes().to_vec())?,
        })
    }
}

impl Persist for VariantConfig {
    fn write(&self, metadata: Metadata) -> Result<(), Vec<u8>> {
        let write_ = |to_file| {
            serde_json::to_writer(&self.write_path, to_file)
                .map_err(|e| e.to_string().as_bytes().to_vec())
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

    fn read(&self, username: String) -> Result<Option<Metadata>, Vec<u8>> {
        Ok(self
            .read_all()?
            .into_iter()
            .find(|m| m.username == username))
    }

    fn read_all(&self) -> Result<Vec<Metadata>, Vec<u8>> {
        serde_json::from_reader::<File, Vec<Metadata>>(
            self.write_path
                .try_clone()
                .map_err(|e| e.to_string().as_bytes().to_vec())?,
        )
        .map_err(|e| e.to_string().as_bytes().to_vec())
    }
}
