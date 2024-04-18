use libsecp256k1::{PublicKey, Signature};
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::hash;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Job {
    pub reward: u32,
    pub num_of_steps: u32,
    pub cairo_pie: Vec<u8>,
    pub public_key: PublicKey,
    pub signature: Signature,
    // below fields not bounded by signature
    pub cpu_air_params: Vec<u8>,        // needed for proving
    pub cpu_air_prover_config: Vec<u8>, // needed for proving
}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reward.hash(state);
        self.num_of_steps.hash(state);
        self.cairo_pie.hash(state);
        self.cpu_air_prover_config.hash(state);
        self.cpu_air_params.hash(state);
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(hash!(self).to_be_bytes()))
    }
}
