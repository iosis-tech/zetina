use crate::hash;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

/*
    Job Witness Object
    This object represents the output from the proving process.
    It holds a serialized proof as an array of bytes.
    This serialized proof can be deserialized into a StarkProof object by the verifier to proceed with the verification of the statement.
*/

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct JobWitness {
    pub job_hash: u64,
    pub proof: Vec<u8>,
}

impl Hash for JobWitness {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.proof.hash(state);
    }
}

impl Display for JobWitness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(hash!(self).to_be_bytes()))
    }
}
