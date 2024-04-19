use crate::hash;
use cairo_felt::Felt252;
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct JobWitness {
    pub proof: Vec<Felt252>,
}

impl Display for JobWitness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(hash!(self).to_be_bytes()))
    }
}
