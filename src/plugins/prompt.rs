use inquire::Text;

use super::Metadata;
use crate::errors::VariantError;

pub fn input_prompt(username: String) -> Result<Metadata, VariantError> {
    Ok(Metadata {
        name: match Text::new("git metadata name to set in config (e.g. John Doe)?").prompt() {
            Ok(n) => n,
            Err(e) => return Err(VariantError::IO(e.to_string())),
        },
        email: match Text::new("git metadata to set in config?").prompt() {
            Ok(e) => e,
            Err(e) => return Err(VariantError::IO(e.to_string())),
        },
        username,
    })
}
