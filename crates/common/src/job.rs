use crate::hash;
use serde::{Deserialize, Serialize};
use starknet::{
    core::types::FromByteSliceError,
    signers::{SigningKey, VerifyingKey},
};
use starknet_crypto::{poseidon_hash_many, FieldElement, Signature};
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
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_data: JobData,
    pub public_key: FieldElement, // The public key of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
    pub signature_r: FieldElement, // The signature of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
    pub signature_s: FieldElement, // The signature of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
}

impl Job {
    pub fn try_from_job_data(job_data: JobData, signing_key: &SigningKey) -> Self {
        let message_hash: FieldElement = job_data.poseidon_hash();
        let signature = signing_key.sign(&message_hash).unwrap();
        let public_key = signing_key.verifying_key().scalar();
        Self { job_data, public_key, signature_r: signature.r, signature_s: signature.s }
    }

    pub fn verify_signature(&self) -> bool {
        let message_hash: FieldElement = self.job_data.poseidon_hash();
        VerifyingKey::from_scalar(self.public_key)
            .verify(&message_hash, &Signature { r: self.signature_r, s: self.signature_s })
            .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JobData {
    pub reward: u32,
    pub num_of_steps: u32,
    #[serde(with = "chunk_felt_array")]
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

    pub fn poseidon_hash(&self) -> FieldElement {
        poseidon_hash_many(&chunk_felt_array::from_data_vec_to_vec_field_elements(
            &self.cairo_pie_compressed,
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn job_verify_signature(job in any::<Job>()) {
            assert!(job.verify_signature());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        #[test]
        fn job_serialization(job in any::<Job>()) {
            let serialized_job = serde_json::to_string(&job).unwrap();
            let deserialized_job: Job = serde_json::from_str(&serialized_job).unwrap();
            assert_eq!(job, deserialized_job)
        }
    }
}

mod chunk_felt_array {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use starknet_crypto::FieldElement;

    const FIELD_ELEMENT_SIZE: usize = 32;
    const FIELD_ELEMENT_CHUNK_SIZE: usize = 31;

    #[derive(Serialize, Deserialize)]
    struct FieldElementsData {
        data_len: usize,
        data: Vec<FieldElement>,
    }

    pub fn from_data_vec_to_vec_field_elements(value: &[u8]) -> Vec<FieldElement> {
        value
            .chunks(FIELD_ELEMENT_CHUNK_SIZE)
            .map(|chunk| FieldElement::from_byte_slice_be(chunk).unwrap())
            .collect()
    }

    pub fn from_field_elements_vec_to_data_vec(value: &[FieldElement], len: usize) -> Vec<u8> {
        let mut v: Vec<u8> = vec![];

        if let Some((last, elements)) = value.split_last() {
            v.extend(elements.iter().flat_map(|felt| {
                felt.to_bytes_be().as_slice()
                    [(FIELD_ELEMENT_SIZE - FIELD_ELEMENT_CHUNK_SIZE)..FIELD_ELEMENT_SIZE]
                    .to_vec()
            }));
            v.extend(
                last.to_bytes_be().as_slice()[(FIELD_ELEMENT_CHUNK_SIZE * value.len() - len
                    + FIELD_ELEMENT_SIZE
                    - FIELD_ELEMENT_CHUNK_SIZE)
                    ..FIELD_ELEMENT_SIZE]
                    .to_vec(),
            )
        };

        v
    }

    pub fn serialize<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        FieldElementsData {
            data_len: value.len(),
            data: from_data_vec_to_vec_field_elements(value),
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let field_elements_data = FieldElementsData::deserialize(deserializer)?;
        Ok(from_field_elements_vec_to_data_vec(
            &field_elements_data.data,
            field_elements_data.data_len,
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(10000))]
            #[test]
            fn data_to_felt_to_data(data in any_with::<Vec<u8>>(((0..100).into(), ()))) {
                let felts = from_data_vec_to_vec_field_elements(&data);
                assert_eq!(data, from_field_elements_vec_to_data_vec(&felts, data.len()))
            }
        }
    }
}
