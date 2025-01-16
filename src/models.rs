use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Envelope {
    pub to: Vec<String>,
    pub from: String,
}

#[derive(Deserialize)]
pub struct QueryParameters {
    pub passphrase: String,
}
