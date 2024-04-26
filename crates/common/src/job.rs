use crate::hash;
use serde::{Deserialize, Serialize};
use starknet::core::types::FromByteSliceError;
use starknet_crypto::FieldElement;
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
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_data: JobData,
    pub public_key: Vec<u8>, // The public key of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
    pub signature: Vec<u8>, // The signature of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
}

impl Job {
    pub fn from_job_data(job_data: JobData, key_pair: libp2p::identity::ecdsa::Keypair) -> Self {
        let message: Vec<u8> = job_data.to_owned().try_into().unwrap();
        let public_key = key_pair.public();
        let signature = key_pair.sign(&message);
        Self { job_data, public_key: public_key.to_bytes(), signature }
    }

    pub fn verify_signature(&self) -> bool {
        let message: Vec<u8> = self.job_data.to_owned().try_into().unwrap();
        let public_key =
            libp2p::identity::ecdsa::PublicKey::try_from_bytes(self.public_key.as_slice()).unwrap();
        let signature = &self.signature;
        public_key.verify(&message, signature)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JobData {
    pub reward: u32,
    pub num_of_steps: u32,
    pub cairo_pie_compressed: Vec<u8>,
    pub registry_address: FieldElement,
}

impl JobData {
    pub fn new(
        reward: u32,
        num_of_steps: u32,
        cairo_pie_compressed: Vec<u8>,
        registry_address: FieldElement,
    ) -> Self {
        Self { reward, num_of_steps, cairo_pie_compressed, registry_address }
    }
}

impl TryFrom<JobData> for Vec<u8> {
    type Error = FromByteSliceError;
    fn try_from(value: JobData) -> Result<Self, Self::Error> {
        let mut felts: Vec<FieldElement> =
            vec![FieldElement::from(value.reward), FieldElement::from(value.num_of_steps)];
        felts.extend(
            value
                .cairo_pie_compressed
                .chunks(31)
                .map(|chunk| FieldElement::from_byte_slice_be(chunk).unwrap()),
        );
        felts.push(value.registry_address);
        Ok(felts.iter().flat_map(|felt| felt.to_bytes_be()).collect())
    }
}

impl TryFrom<JobData> for Vec<FieldElement> {
    type Error = FromByteSliceError;
    fn try_from(value: JobData) -> Result<Self, Self::Error> {
        let mut felts: Vec<FieldElement> =
            vec![FieldElement::from(value.reward), FieldElement::from(value.num_of_steps)];
        felts.extend(
            value
                .cairo_pie_compressed
                .chunks(31)
                .map(|chunk| FieldElement::from_byte_slice_be(chunk).unwrap()),
        );
        felts.push(value.registry_address);
        Ok(felts)
    }
}

impl Hash for JobData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reward.hash(state);
        self.num_of_steps.hash(state);
        self.cairo_pie_compressed.hash(state);
        self.registry_address.hash(state);
    }
}

impl Hash for Job {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.job_data.hash(state)
    }
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(hash!(self).to_be_bytes()))
    }
}
