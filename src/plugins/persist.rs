use std::{fs::OpenOptions, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::{Metadata, Persist};
use crate::errors::VariantError;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectMapping {
    remote_url: String,
    profile_username: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct CacheData {
    #[serde(default)]
    profiles: Vec<Metadata>,
    #[serde(default)]
    projects: Vec<ProjectMapping>,
}

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

        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&variant_file)?;

        Ok(Self {
            write_path: variant_file,
        })
    }

    pub fn cache_project(
        &self,
        remote_url: String,
        profile_username: String,
    ) -> Result<(), VariantError> {
        let mut data = self.read_cache()?;

        if let Some(existing) = data
            .projects
            .iter_mut()
            .find(|p| p.remote_url == remote_url)
        {
            existing.profile_username = profile_username;
        } else {
            data.projects.push(ProjectMapping {
                remote_url,
                profile_username,
            });
        }

        self.write_cache(&data)
    }

    pub fn get_project_profile(&self, remote_url: &str) -> Result<Option<String>, VariantError> {
        let data = self.read_cache()?;
        Ok(data
            .projects
            .iter()
            .find(|p| p.remote_url == remote_url)
            .map(|p| p.profile_username.clone()))
    }

    fn read_cache(&self) -> Result<CacheData, VariantError> {
        let content = std::fs::read_to_string(&self.write_path)?;
        match serde_json::from_str::<CacheData>(&content) {
            Ok(data) => return Ok(data),
            Err(err) => {
                if err.is_eof() {
                    return Ok(CacheData::default());
                }
            }
        };

        let profiles: Result<Vec<Metadata>, _> = serde_json::from_str(&content);
        match profiles {
            Ok(profiles) => Ok(CacheData {
                profiles,
                projects: Vec::new(),
            }),
            Err(_) => Ok(CacheData::default()),
        }
    }

    fn write_cache(&self, data: &CacheData) -> Result<(), VariantError> {
        serde_json::to_writer(OpenOptions::new().write(true).open(&self.write_path)?, data)
            .map_err(|e| e.into())
    }
}

impl Persist for VariantConfig {
    fn write(&self, metadata: Metadata) -> Result<(), VariantError> {
        let mut data = self.read_cache()?;

        if let Some(existing) = data
            .profiles
            .iter_mut()
            .find(|m| m.username == metadata.username)
        {
            existing.name = metadata.name;
            existing.email = metadata.email;
        } else {
            data.profiles.push(metadata);
        }

        self.write_cache(&data)
    }

    fn read(&self, username: String) -> Result<Option<Metadata>, VariantError> {
        let data = self.read_cache()?;
        Ok(data.profiles.into_iter().find(|m| m.username == username))
    }

    fn read_all(&self) -> Result<Vec<Metadata>, VariantError> {
        let data = self.read_cache()?;
        Ok(data.profiles)
    }
}
