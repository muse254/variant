use super::Metadata;

#[derive(Default)]
pub(crate) struct Prompt;

impl Prompt {
    pub fn plugin(_name: String) -> Result<Metadata, Vec<u8>> {
        todo!()
    }
}
