use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

use cairo_felt::Felt252;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct JobWitness {
    pub data: Vec<Felt252>,
}

impl Display for JobWitness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        write!(f, "{}", hex::encode(hasher.finish().to_be_bytes()))
    }
}
