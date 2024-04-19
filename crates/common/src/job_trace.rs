use crate::hash;
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};
use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct JobTrace {
    pub air_public_input: NamedTempFile,
    pub air_private_input: NamedTempFile,
    pub memory: NamedTempFile, // this is not used directly but needs to live for air_private_input to be valid
    pub trace: NamedTempFile, // this is not used directly but needs to live for air_private_input to be valid
    pub cpu_air_prover_config: NamedTempFile,
    pub cpu_air_params: NamedTempFile,
}

impl Hash for JobTrace {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.air_public_input.path().hash(state);
        self.air_private_input.path().hash(state);
        self.memory.path().hash(state);
        self.trace.path().hash(state);
        self.cpu_air_prover_config.path().hash(state);
        self.cpu_air_params.path().hash(state);
    }
}

impl Display for JobTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(hash!(self).to_be_bytes()))
    }
}
