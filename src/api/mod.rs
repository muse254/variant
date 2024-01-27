use std::{io, path::Path, process::Command};

use anyhow::Result;

use crate::plugins::{KeyPair, Metadata, Persist, Variant};

/// whoami returns the git profile information for the current configuration.
pub fn whoami(verbose: bool) -> Result<Vec<u8>, Vec<u8>> {
    let data = Command::new("git")
        .args(["config", "--list"])
        .output()
        .map_err(|e| e.to_string().as_bytes().to_vec())?;

    if !data.status.success() {
        return Err(data.stdout);
    }

    if verbose {
        return Ok(data.stdout);
    }

    // Only keeping the lines we care about.
    const KEYS: [&[u8]; 3] = [b"user.name", b"user.email", b"user.signingkey"];

    let mut truncated = Vec::new();
    for line in data.stdout.split(|&c| c == b'\n') {
        for key in KEYS.iter() {
            if line.starts_with(key) {
                truncated.extend_from_slice(b"\n");
                truncated.extend_from_slice(line);
            }
        }
    }

    Ok(truncated)
}

/// Provides the list of all git profile variants found, following the convention.
pub fn variants() -> Result<Vec<Variant>, Vec<u8>> {
    let config_root = home::home_dir()
        .ok_or_else(|| b"cannot find home directory")?
        .join(".ssh");

    if !config_root.exists() || !config_root.is_dir() {
        return Err(b"cannot find ssh config directory".to_vec());
    }

    let mut variants = Vec::new();
    for entry in config_root
        .read_dir()
        .expect("must be able to read ssh config directory")
    {
        let entry = entry.map_err(|e| e.to_string().as_bytes().to_vec())?;
        let path = entry.path();
        if path.is_dir() {
            let name = path
                .file_name()
                .ok_or_else(|| b"invalid path")?
                .to_str()
                .ok_or_else(|| b"invalid path")?
                .to_string();

            variants.push(Variant {
                name,
                keys: keys(&path)?,
            });
        }
    }

    Ok(variants)
}

/// Sets the git profile variant. If sacred is true, then only the local config
/// will be changed and the global config remains untouched; the inverse is true otherwise.
/// The name must be the name of the profile to use. e.g. `foo` or `bar` depending on the
/// folder the config is in.
///
/// The provider is triggered if the metadata cannot be found in the cache.
pub fn set_variant<Cache: Persist>(
    name: String,
    cache: Cache,
    provider: &dyn Fn(String) -> Result<Metadata, Vec<u8>>,
    sacred: bool,
) -> Result<(), Vec<u8>> {
    let variant = variants()?
        .into_iter()
        .find(|v| v.name == name)
        .ok_or_else(|| b"cannot find variant".to_vec())?;

    let start_agent = Command::new("ssh-agent")
        .arg("-s")
        .output()
        .map_err(|e| e.to_string().as_bytes().to_vec())?;

    if !start_agent.status.success() {
        return Err(start_agent.stdout);
    }

    let register_key = Command::new("ssh-add")
        .arg(variant.keys.1.to_str().expect("must be valid utf-8"))
        .stderr(io::stderr())
        .stdout(io::stdout())
        .output()
        .map_err(|e| e.to_string().as_bytes().to_vec())?;

    if !register_key.status.success() {
        return Err(register_key.stdout);
    }

    let metadata = match cache.read(name.clone())? {
        Some(metadata) => metadata,
        None => {
            let data = provider(name)?;
            cache.write(data.clone())?;
            data
        }
    };

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
            .map_err(|e| e.to_string().as_bytes().to_vec())?;

        if !output.status.success() {
            return Err(output.stdout);
        }
    }

    // log changes
    println!("{}\n", String::from_utf8_lossy(&whoami(false)?));

    Ok(())
}

/// keys returns the public key and private key pair, respectively.
fn keys(path: &Path) -> Result<KeyPair, Vec<u8>> {
    // the algorithm is rudimentary, works for now and there's no need to over-engineer it:
    // - no key has the name `config`
    // - the private key and public key have the same name with the latter having a `.pub` extension
    for entry in path
        .read_dir()
        .map_err(|e| e.to_string().as_bytes().to_vec())?
    {
        let entry = entry.map_err(|e| e.to_string().as_bytes().to_vec())?;
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_name()
                .ok_or_else(|| b"invalid path")?
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

    Err(b"cannot find private key".to_vec())
}
