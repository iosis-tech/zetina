use rand::{thread_rng, Rng};
use sharp_p2p_common::job::Job;
use starknet::providers::sequencer::models::L1Address;
use std::{env, path::PathBuf};

pub struct TestFixture {
    pub job: Job,
    pub program_path: PathBuf,
}

pub fn fixture() -> TestFixture {
    let mut rng = thread_rng();
    let ws_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"))
            .join("../../");
    let cairo_pie_path = ws_root.join("crates/tests/cairo/fibonacci_pie.zip");
    let program_path = ws_root.join("target/bootloader.json");

    TestFixture {
        job: Job::new(
            rng.gen(),
            rng.gen(),
            cairo_pie_path,
            L1Address::random(),
            libsecp256k1::SecretKey::random(&mut rng),
        ),
        program_path,
    }
}
