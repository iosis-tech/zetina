use crate::hash;
use libsecp256k1::{curve::Scalar, sign, Message, PublicKey, SecretKey, Signature};
use std::{
    fmt::Display,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

/*
    Job Object
    This object represents a task requested by a delegator.
    It contains metadata that allows the executor to decide if the task is attractive enough to run.
    It includes a pie object that holds the task bytecode itself.
    Additionally, the object holds the signature and public key of the delegator, enabling the executor to prove to the Registry that the task was intended by the delegator.
    The Job object also includes the target registry where the delegator expects this proof to be verified.
*/

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Job {
    pub reward: u32,              // The reward offered for completing the task
    pub num_of_steps: u32, // The number of steps expected to complete the task (executor ensures that this number is greater than or equal to the actual steps; in the future, the executor may charge a fee to the delegator if not met)
    pub cairo_pie: Vec<u8>, // The task bytecode in compressed zip format, to conserve memory
    pub registry_address: String, // The address of the registry contract where the delegator expects the proof to be verified
    pub public_key: PublicKey, // The public key of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
    pub signature: Signature, // The signature of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
}

impl Job {
    pub fn new(
        reward: u32,
        num_of_steps: u32,
        cairo_pie_file: PathBuf,
        registry_address: &str,
        secret_key: SecretKey,
    ) -> Self {
        Self {
            reward,
            num_of_steps,
            cairo_pie: fs::read(cairo_pie_file).unwrap(),
            registry_address: registry_address.to_string(),
            public_key: PublicKey::from_secret_key(&secret_key),
            signature: libsecp256k1::sign(
                // TODO proper impl just mocked rn for tests
                &Message::parse(&[
                    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                ]),
                &secret_key,
            )
            .0,
        }
    }
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
            registry_address: "0x0".to_string(),
        }
    }
}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reward.hash(state);
        self.num_of_steps.hash(state);
        self.cairo_pie.hash(state);
        self.registry_address.hash(state);
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
