use libsecp256k1::{curve::Scalar, sign, PublicKey, SecretKey, Signature};
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

impl Default for Job {
    fn default() -> Self {
        let secret_key = &SecretKey::default();
        let public_key = PublicKey::from_secret_key(secret_key);
        let (signature, _recovery_id) =
            sign(&libsecp256k1::Message(Scalar([0, 0, 0, 0, 0, 0, 0, 0])), secret_key);
        Self {
            reward: 0,
            num_of_steps: 0,
            cairo_pie: vec![1, 2, 3],
            public_key,
            signature,
            cpu_air_params: vec![1, 2, 3],
            cpu_air_prover_config: vec![1, 2, 3],
        }
    }
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

// impl Job {
//     pub fn serialize_job(&self) -> Vec<u8> {
//         bincode::serialize(self).unwrap()
//     }

//     pub fn deserialize_job(serialized_job: &[u8]) -> Self {
//         bincode::deserialize(serialized_job).unwrap()
//     }
// }
