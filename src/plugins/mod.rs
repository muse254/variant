use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Persist represents a persistence layer for the metadata; it can be a file, a database, etc.
/// The implementation is left to the user.
pub trait Persist {
    fn write(&self, metadata: Metadata) -> Result<(), Vec<u8>>;
    fn read(&self, username: String) -> Result<Option<Metadata>, Vec<u8>>;
    fn read_all(&self) -> Result<Vec<Metadata>, Vec<u8>>;
}

/// Metadata represents the git profile metadata.
#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    // Refers to the fully qualified e.g. like John Doe.
    pub name: String,
    // Refers to the email address e.g. john.doe@example.
    pub email: String,
    // Refers to the username e.g. "foo".
    pub username: String,
}

/// Variant represents a git profile variant,
pub struct Variant {
    /// The name represents the folder path as per the convention of this project.
    pub name: String,
    /// The path to the private key.
    pub(crate) keys: KeyPair,
}

/// The public key and private key pair, respectively.
pub type KeyPair = (PathBuf, PathBuf);

mod persist;
mod prompt;
pub(crate) use persist::VariantConfig;
pub(crate) use prompt::Prompt;
