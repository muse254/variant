use anyhow::Result;
use std::process::Command;

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
