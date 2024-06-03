use serde::{Deserialize, Serialize};
use starknet_crypto::FieldElement;
use zetina_common::job::Job;

#[derive(Serialize, Deserialize)]
pub struct SimpleBootloaderInput {
    pub public_key: FieldElement,
    pub job: Job,
    pub single_page: bool,
}
