use libsecp256k1::PublicKey;
use serde::{Deserialize, Serialize};
use sharp_p2p_common::job::Job;

#[derive(Serialize, Deserialize)]
pub struct SimpleBootloaderInput {
    pub identity: PublicKey,
    pub job: Job,
    pub single_page: bool,
}
