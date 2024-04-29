use serde::{de::Visitor, Deserialize};
use serde_json::Value;
use starknet_crypto::FieldElement;
use std::{ops::Deref, str::FromStr};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArgVecError {
    #[error("failed to parse number: {0}")]
    NumberParseError(#[from] std::num::ParseIntError),
    #[error("failed to parse bigint: {0}")]
    BigIntParseError(#[from] num_bigint::ParseBigIntError),
    #[error("failed to parse field_element: {0}")]
    FieldElementParseError(#[from] starknet::core::types::FromByteSliceError),
    #[error("number out of range")]
    NumberOutOfRange,
}

/// `ArgVec` is a wrapper around a vector of `Arg`.
///
/// It provides convenience methods for working with a vector of `Arg` and implements
/// `Deref` to allow it to be treated like a vector of `Arg`.
#[derive(Debug, Clone)]
pub struct ArgVec(Vec<FieldElement>);

impl ArgVec {
    #[must_use]
    pub fn new(args: Vec<FieldElement>) -> Self {
        Self(args)
    }
}

impl IntoIterator for ArgVec {
    type Item = FieldElement;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for ArgVec {
    type Target = Vec<FieldElement>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ArgVec> for Vec<FieldElement> {
    fn from(args: ArgVec) -> Self {
        args.0
    }
}

impl From<Vec<FieldElement>> for ArgVec {
    fn from(args: Vec<FieldElement>) -> Self {
        Self(args)
    }
}

impl ArgVec {
    fn visit_seq_helper(seq: &[Value]) -> Result<Self, ArgVecError> {
        let iterator = seq.iter();
        let mut args = Vec::new();

        for arg in iterator {
            match arg {
                Value::Number(n) => {
                    let n = n.as_u64().ok_or(ArgVecError::NumberOutOfRange)?;
                    args.push(FieldElement::from(n));
                }
                Value::String(n) => {
                    let n = num_bigint::BigUint::from_str(n)?;
                    args.push(
                        FieldElement::from_byte_slice_be(&n.to_bytes_be())
                            .map_err(ArgVecError::FieldElementParseError)?,
                    );
                }
                Value::Array(a) => {
                    args.push(FieldElement::from(a.len()));
                    let result = Self::visit_seq_helper(a)?;
                    args.extend(result.0);
                }
                _ => (),
            }
        }

        Ok(Self::new(args))
    }
}

impl<'de> Visitor<'de> for ArgVec {
    type Value = ArgVec;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a list of arguments")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut args = Vec::new();
        while let Some(arg) = seq.next_element()? {
            match arg {
                Value::Number(n) => args.push(Value::Number(n)),
                Value::String(n) => args.push(Value::String(n)),
                Value::Array(a) => args.push(Value::Array(a)),
                _ => return Err(serde::de::Error::custom("Invalid type")),
            }
        }

        Self::visit_seq_helper(&args).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for ArgVec {
    fn deserialize<D>(deserializer: D) -> Result<ArgVec, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ArgVec(Vec::new()))
    }
}
