use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Job {
    pub reward: u32,
    pub num_of_steps: u32,
    pub private_input: Vec<u8>,
    pub public_input: Vec<u8>,
    pub cpu_air_prover_config: Vec<u8>,
    pub cpu_air_params: Vec<u8>,
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        write!(f, "{}", hex::encode(hasher.finish().to_be_bytes()))
    }
}
