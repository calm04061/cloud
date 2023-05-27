use serde_derive::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Callback {
    pub(crate) code: String,
    pub(crate) state: String,
}
