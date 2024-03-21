use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Callback {
    pub(crate) code: String,
    pub(crate) state: String,
}
