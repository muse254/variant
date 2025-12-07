//! The API module provides the core functionality of the application.

use std::{io, path::Path, process::Command};

use anyhow::Result;

use crate::errors::VariantError;
use crate::plugins::persist::VariantConfig;
use crate::plugins::{KeyPair, Metadata, Persist, Variant};

/// whoami returns the git profile information for the current configuration.
pub fn whoami(verbose: bool) -> Result<Vec<u8>, VariantError> {
    let is_repo_cached = || -> bool {
        if let Ok(Some(remote_url)) = get_git_remote_url()
            && let Ok(project_cache) = VariantConfig::init()
        {
            return project_cache.get_project_profile(&remote_url).is_ok();
        }
        false
    };

    if verbose {
        let data = Command::new("git").args(["config", "--list"]).output()?;

        if !data.status.success() {
            return Err(VariantError::Shell(
                String::from_utf8_lossy(&data.stdout).into(),
            ));
        }

        return Ok(data.stdout);
    }

    const KEYS: [&str; 3] = ["user.name", "user.email", "user.signingkey"];
    let use_local = is_repo_cached();
    let config_flag = if use_local { "--local" } else { "--global" };

    let mut truncated = Vec::new();
    let mut first = true;
    for key in KEYS.iter() {
        let output = Command::new("git")
            .args(["config", config_flag, "--get", key])
            .output()?;

        if output.status.success() && !output.stdout.is_empty() {
            if !first {
                truncated.extend_from_slice(b"\n");
            }
            truncated.extend_from_slice(key.as_bytes());
            truncated.extend_from_slice(b"=");
            let value = output.stdout.strip_suffix(b"\n").unwrap_or(&output.stdout);
            truncated.extend_from_slice(value);
            first = false;
        }
    }

    Ok(truncated)
}

/// Provides the list of all git profile variants found, following the convention.
pub fn variants() -> Result<Vec<Variant>, VariantError> {
    let config_root = home::home_dir()
        .ok_or_else(|| VariantError::IO("cannot find home directory".into()))?
        .join(".ssh");

    if !config_root.exists() || !config_root.is_dir() {
        return Err(VariantError::IO("cannot find ssh config directory".into()));
    }

    let mut variants = Vec::new();
    for entry in config_root.read_dir()? {
        let path = entry?.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .ok_or_else(|| VariantError::IO("invalid path".into()))?
                .to_str()
                .ok_or_else(|| VariantError::IO("invalid path".into()))?
                .to_string();

            variants.push(Variant {
                name,
                keys: keys(&path)?,
            });
        }
    }

    Ok(variants)
}

pub fn get_git_remote_url() -> Result<Option<String>, VariantError> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        Ok(None)
    } else {
        Ok(Some(url))
    }
}

/// Sets the git profile variant. If sacred is true, then only the local config
/// will be changed and the global config remains untouched; the inverse is true otherwise.
/// When sacred is true, the project will be cached and tied to this profile.
/// The name must be the name of the profile to use. e.g. `foo` or `bar` depending on the
/// folder the config is in.
///
/// The provider is triggered if the metadata cannot be found in the cache.
pub fn set_variant<Cache: Persist>(
    name: String,
    cache: Cache,
    provider: &dyn Fn(String) -> Result<Metadata, VariantError>,
    sacred: bool,
) -> Result<(), VariantError> {
    let start_agent = Command::new("ssh-agent").arg("-s").output()?;
    if !start_agent.status.success() {
        return Err(VariantError::Shell(
            String::from_utf8_lossy(&start_agent.stdout).into(),
        ));
    }

    let variant = variants()?
        .into_iter()
        .find(|v| v.name == name)
        .ok_or_else(|| VariantError::System("cannot find variant".into()))?;

    let clear_identities = Command::new("ssh-add")
        .arg("-D")
        .stdout(io::stdout())
        .output()
        .map_err(|e| VariantError::Shell(e.to_string()))?;

    if !clear_identities.status.success() {
        return Err(VariantError::Shell(
            String::from_utf8_lossy(&clear_identities.stdout).into(),
        ));
    }

    let register_key = Command::new("ssh-add")
        .arg(variant.keys.1.to_str().expect("must be valid utf-8"))
        .stderr(io::stderr())
        .stdout(io::stdout())
        .output()
        .map_err(|e| VariantError::Shell(e.to_string()))?;

    if !register_key.status.success() {
        return Err(VariantError::Shell(
            String::from_utf8_lossy(&register_key.stdout).into(),
        ));
    }

    let metadata = match cache.read(name.clone())? {
        Some(metadata) => metadata,
        None => {
            let data = provider(name.clone())?;
            cache.write(data.clone())?;
            data
        }
    };

    if let Some(remote_url) = get_git_remote_url()?
        && sacred
        && let Ok(project_cache) = VariantConfig::init()
    {
        let _ = project_cache.cache_project(remote_url.clone(), name.clone());
    } else if sacred {
        return Err(VariantError::System(
            "Cannot use --sacred flag: not in a git repository with a remote origin.".into(),
        ));
    }

    for pair in [
        ("user.name", metadata.name),
        ("user.email", metadata.email),
        (
            "user.signingkey",
            String::from(variant.keys.0.to_str().expect("must be valid utf-8")),
        ),
    ] {
        let output = Command::new("git")
            .arg("config")
            .arg(if sacred { "--local" } else { "--global" })
            .arg(pair.0)
            .arg(pair.1)
            .output()
            .map_err(|e| VariantError::Shell(e.to_string()))?;

        if !output.status.success() {
            return Err(VariantError::Shell(
                String::from_utf8_lossy(&output.stdout).into(),
            ));
        }
    }

    println!("{}\n", String::from_utf8_lossy(&whoami(false)?));

    Ok(())
}

/// keys returns the public key and private key pair, respectively.
fn keys(path: &Path) -> Result<KeyPair, VariantError> {
    // the algorithm is rudimentary, works for now and there's no need to over-engineer it:
    // - no key has the name `config`
    // - the private key and public key have the same name with the latter having a `.pub` extension
    for entry in path.read_dir()? {
        let path = entry?.path();
        if path.is_file() {
            let name = path
                .file_name()
                .ok_or_else(|| VariantError::IO("invalid path".into()))?
                .to_str()
                .expect("file name must be valid utf-8");

            if name.eq("config") {
                continue;
            }

            if name.ends_with(".pub") {
                let partner = path
                    .parent()
                    .expect("must have parent")
                    .join(name.strip_suffix(".pub").expect("must have suffix"));

                return Ok((path, partner));
            }
        }
    }

    Err(VariantError::IO("cannot find private key".into()))
}
