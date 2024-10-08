use crate::hash;
use libp2p::kad;
use std::{
    fmt::Display,
    hash::{DefaultHasher, Hash, Hasher},
    mem::ManuallyDrop,
};
use tempfile::NamedTempFile;

/*
    Job Trace Object
    This object represents the output from the Cairo run process in proof mode.
    It includes objects such as public input, private input, trace, and memory.
*/

#[derive(Debug)]
pub struct JobTrace {
    pub job_key: kad::RecordKey,
    pub air_public_input: NamedTempFile, // Temporary file containing the public input
    pub air_private_input: NamedTempFile, // Temporary file containing the private input; memory and trace files must exist for this to be valid
    pub memory: ManuallyDrop<NamedTempFile>, // Temporary file containing memory data (required for air_private_input validity)
    pub trace: ManuallyDrop<NamedTempFile>, // Temporary file containing trace data (required for air_private_input validity)
}

impl JobTrace {
    pub fn new(
        job_key: kad::RecordKey,
        air_public_input: NamedTempFile,
        air_private_input: NamedTempFile,
        memory: NamedTempFile,
        trace: NamedTempFile,
    ) -> Self {
        Self {
            job_key,
            air_public_input,
            air_private_input,
            memory: ManuallyDrop::new(memory),
            trace: ManuallyDrop::new(trace),
        }
    }
}

impl Drop for JobTrace {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.memory);
            ManuallyDrop::drop(&mut self.trace);
        }
    }
}

impl Hash for JobTrace {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.air_public_input.path().hash(state);
        self.air_private_input.path().hash(state);
        self.memory.path().hash(state);
        self.trace.path().hash(state);
    }
}

impl Display for JobTrace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(hash!(self).to_be_bytes()))
    }
}
