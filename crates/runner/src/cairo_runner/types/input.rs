use serde::{Deserialize, Serialize};
use sharp_p2p_common::job::Job;
use starknet_crypto::FieldElement;

#[derive(Serialize, Deserialize)]
pub struct SimpleBootloaderInput {
    pub public_key: FieldElement,
    pub job: Job,
    pub single_page: bool,
}
