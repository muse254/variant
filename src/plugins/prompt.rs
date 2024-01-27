use inquire::Text;

use super::Metadata;

pub fn input_prompt(username: String) -> Result<Metadata, Vec<u8>> {
    let name;
    match Text::new("git metadata name to set in config (e.g. John Doe)?").prompt() {
        Ok(n) => name = n,
        Err(e) => return Err(e.to_string().as_bytes().to_vec()),
    }

    let email;
    match Text::new("git metadata to set in config?").prompt() {
        Ok(e) => email = e,
        Err(e) => return Err(e.to_string().as_bytes().to_vec()),
    }

    Ok(Metadata {
        name,
        email,
        username,
    })
}
