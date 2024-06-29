use rand::{thread_rng, Rng};
use zetina_common::job::{Job, JobData};

use starknet::signers::SigningKey;
use std::{env, fs, path::PathBuf};

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
        job: Job::try_from_job_data(
            JobData::new(rng.gen(), fs::read(cairo_pie_path).unwrap()),
            &SigningKey::from_random(),
        ),
        program_path,
    }
}
