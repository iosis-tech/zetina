use libsecp256k1::{curve::Scalar, sign, PublicKey, SecretKey, Signature};
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::hash;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Job {
    pub reward: u32,
    pub num_of_steps: u32, // executor needs to make sure that this num of steps are >= real ones if not executor can charge a fee on delegator in Registry (added in future)
    pub cairo_pie: Vec<u8>, // zip format compressed bytes, no point in inflating it in RAM
    pub public_key: PublicKey, // used it bootloader stage to confirm Job<->Delegator auth
    pub signature: Signature, // used it bootloader stage to confirm Job<->Delegator auth
    // below fields not bounded by signature
    // needed for proving,
    // prover can falsify it but it is executor responsibility so that the proof passes the verifier checks,
    // Delegator is interested only in succesfull proof verification and output of the job
    pub cpu_air_params: Vec<u8>,        // JSON file serialized
    pub cpu_air_prover_config: Vec<u8>, // JSON file serialized
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
