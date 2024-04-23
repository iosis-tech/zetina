use crate::hash;
use libsecp256k1::{Message, PublicKey, SecretKey, Signature};
use proptest::arbitrary::any;
use proptest::prop_compose;
use proptest::strategy::BoxedStrategy;
use proptest::{arbitrary::Arbitrary, strategy::Strategy};
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::types::FromByteSliceError;
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

#[serde_as]
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct Job {
    pub job_data: JobData,
    #[serde_as(as = "[_; 65]")]
    pub public_key: [u8; 65], // The public key of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
    #[serde_as(as = "[_; 64]")]
    pub signature: [u8; 64], // The signature of the delegator, used in the bootloader stage to confirm authenticity of the Job<->Delegator relationship
}

impl Job {
    pub fn from_job_data(job_data: JobData, secret_key: SecretKey) -> Self {
        let felts: Vec<FieldElement> = job_data.to_owned().try_into().unwrap();
        let message = Message::parse(&poseidon_hash_many(&felts).to_bytes_be());
        let (signature, _recovery) = libsecp256k1::sign(&message, &secret_key);

        Self {
            job_data,
            public_key: PublicKey::from_secret_key(&secret_key).serialize(),
            signature: signature.serialize(),
        }
    }

    pub fn verify_signature(&self) -> bool {
        let felts: Vec<FieldElement> = self.job_data.to_owned().try_into().unwrap();
        let message = Message::parse(&poseidon_hash_many(&felts).to_bytes_be());
        let signature = Signature::parse_overflowing(&self.signature);
        let pubkey = PublicKey::parse(&self.public_key).unwrap();
        libsecp256k1::verify(&message, &signature, &pubkey)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JobData {
    pub reward: u32,
    pub num_of_steps: u32,
    pub cairo_pie_compressed: Vec<u8>,
    pub registry_address: L1Address,
}

impl JobData {
    pub fn new(
        reward: u32,
        num_of_steps: u32,
        cairo_pie_compressed: Vec<u8>,
        registry_address: L1Address,
    ) -> Self {
        Self { reward, num_of_steps, cairo_pie_compressed, registry_address }
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
        felts.push(FieldElement::from_byte_slice_be(&value.registry_address.to_fixed_bytes())?);
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

prop_compose! {
    fn arb_state()(
        reward in any::<u32>(),
        num_of_steps in any::<u32>(),
        cairo_pie_compressed in any::<Vec<u8>>(),
        secret_key in any::<[u8; 32]>()
    ) -> (u32, u32, Vec<u8>, [u8; 32]) {
        (reward, num_of_steps, cairo_pie_compressed, secret_key)
    }
}

impl Arbitrary for Job {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    fn arbitrary() -> Self::Strategy {
        let abs_state = arb_state();
        abs_state
            .prop_map(|(reward, num_of_steps, cairo_pie_compressed, secret_key)| {
                Job::from_job_data(
                    JobData {
                        reward,
                        num_of_steps,
                        cairo_pie_compressed,
                        registry_address: L1Address::random(),
                    },
                    libsecp256k1::SecretKey::parse(&secret_key).unwrap(),
                )
            })
            .boxed()
    }
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        Self::arbitrary()
    }
}
