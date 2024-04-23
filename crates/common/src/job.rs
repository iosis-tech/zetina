use crate::hash;
use libsecp256k1::{curve::Scalar, sign, Message, PublicKey, SecretKey};
use serde::Serialize;
use starknet::providers::sequencer::models::L1Address;
use starknet_crypto::{poseidon_hash_many, FieldElement};
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

/*
    Job Object
    This object represents a task requested by a delegator.
    It contains metadata that allows the executor to decide if the task is attractive enough to run.
    It includes a pie object that holds the task bytecode itself.
    Additionally, the object holds the signature and public key of the delegator, enabling the executor to prove to the Registry that the task was intended by the delegator.
    The Job object also includes the target registry where the delegator expects this proof to be verified.
*/

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Job {
    pub reward: u32,                   // The reward offered for completing the task
    pub num_of_steps: u32, // The number of steps expected to complete the task (executor ensures that this number is greater than or equal to the actual steps; in the future, the executor may charge a fee to the delegator if not met)
    pub cairo_pie_compressed: Vec<u8>, // The task bytecode in compressed zip format, to conserve memory
    pub registry_address: Vec<u8>, // The address of the registry contract where the delegator expects the proof to be verified
    pub public_key: Vec<u8>, // The public key of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
    pub signature: Vec<u8>, // The signature of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
}

impl Job {
    pub fn new(
        reward: u32,
        num_of_steps: u32,
        cairo_pie_compressed: Vec<u8>,
        registry_address: L1Address,
        secret_key: SecretKey,
    ) -> Self {
        let mut felts: Vec<FieldElement> =
            vec![FieldElement::from(reward), FieldElement::from(num_of_steps)];
        felts.extend(
            cairo_pie_compressed
                .chunks(31)
                .map(|chunk| FieldElement::from_byte_slice_be(chunk).unwrap()),
        );
        felts.push(FieldElement::from_byte_slice_be(registry_address.as_bytes()).unwrap());

        let message = Message::parse(&poseidon_hash_many(&felts).to_bytes_be());
        let (signature, _recovery) = libsecp256k1::sign(&message, &secret_key);

        Self {
            reward,
            num_of_steps,
            cairo_pie_compressed,
            registry_address: registry_address.to_fixed_bytes().to_vec(),
            public_key: PublicKey::from_secret_key(&secret_key).serialize().to_vec(),
            signature: signature.serialize().to_vec(),
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
            cairo_pie_compressed: vec![],
            registry_address: L1Address::zero().to_fixed_bytes().to_vec(),
            public_key: public_key.serialize().to_vec(),
            signature: signature.serialize().to_vec(),
        }
    }
}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reward.hash(state);
        self.num_of_steps.hash(state);
        self.cairo_pie_compressed.hash(state);
        self.registry_address.hash(state);
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(hash!(self).to_be_bytes()))
    }
}
