use std::{fs::OpenOptions, path::PathBuf};

use super::{Metadata, Persist};

/// Reads and writes to the local cache file to provide persistent storage.
pub struct VariantConfig {
    write_path: PathBuf,
}

const VARIANT_FILE: &str = ".variant";

impl VariantConfig {
    pub fn init() -> Result<Self, Vec<u8>> {
        let variant_file = home::home_dir()
            .ok_or_else(|| b"cannot find home directory")?
            .join(VARIANT_FILE);

        // making sure the file exists, if not create it
        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&variant_file)
            .map_err(|e| e.to_string().as_bytes().to_vec())?;

        Ok(Self {
            write_path: variant_file,
        })
    }
}

impl Persist for VariantConfig {
    fn write(&self, metadata: Metadata) -> Result<(), Vec<u8>> {
        let write_ = |to_file| {
            let mut file = OpenOptions::new()
                .write(true)
                .open(&self.write_path)
                .map_err(|e| e.to_string().as_bytes().to_vec())?;
            serde_json::to_writer(&mut file, to_file).map_err(|e| e.to_string().as_bytes().to_vec())
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
        match serde_json::from_str::<Vec<Metadata>>(
            &std::fs::read_to_string(&self.write_path)
                .map_err(|e| e.to_string().as_bytes().to_vec())?,
        ) {
            Ok(data) => Ok(data),
            Err(e) => {
                if e.is_eof() {
                    Ok(Vec::new())
                } else {
                    Err(e.to_string().as_bytes().to_vec())
                }
            }
        }
    }
}
