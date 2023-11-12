use clap::error::Result;

pub fn set_profile(name: &str) -> Result<()> {
    if name.is_empty() {
        return set_current_variant();
    }
    todo!("set_profile")
}

fn set_current_variant() -> Result<()> {
    todo!("get_current_profile")
}

pub fn bake_variant(name: &str) -> Result<()> {
    if name.is_empty() {
        return bake_current_variant();
    }
    todo!("bake_current_profile")
}

fn bake_current_variant() -> Result<()> {
    todo!("bake_current_profile")
}
