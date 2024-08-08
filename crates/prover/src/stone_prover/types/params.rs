use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hash {
    Pedersen,
    Poseidon3,
    Keccak256,
    Keccak256Masked160Lsb,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Field {
    PrimeField0,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    pub page_hash: Hash,
}

impl Default for Statement {
    fn default() -> Self {
        Self { page_hash: Hash::Pedersen }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Default, Deserialize)]
pub struct Fri {
    pub fri_step_list: Vec<u64>,
    pub last_layer_degree_bound: u64,
    pub n_queries: u64,
    pub proof_of_work_bits: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default, Deserialize)]
pub struct Stark {
    pub fri: Fri,
    pub log_n_cosets: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Params {
    pub field: Field,
    pub channel_hash: Hash,
    pub commitment_hash: Hash,
    pub n_verifier_friendly_commitment_layers: u64,
    pub pow_hash: Hash,
    pub statement: Statement,
    pub stark: Stark,
    pub use_extension_field: bool,
    pub verifier_friendly_channel_updates: bool,
    pub verifier_friendly_commitment_hash: Hash,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            field: Field::PrimeField0,
            channel_hash: Hash::Poseidon3,
            commitment_hash: Hash::Keccak256Masked160Lsb,
            n_verifier_friendly_commitment_layers: 0,
            pow_hash: Hash::Keccak256,
            statement: Statement::default(),
            stark: Stark::default(),
            use_extension_field: false,
            verifier_friendly_channel_updates: true,
            verifier_friendly_commitment_hash: Hash::Poseidon3,
        }
    }
}
