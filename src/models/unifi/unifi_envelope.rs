use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UnifiEnvelope<T> {
    pub data: Vec<T>,
}
