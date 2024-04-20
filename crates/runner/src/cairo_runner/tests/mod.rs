use super::CairoRunner;
use crate::traits::RunnerController;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use rand::{thread_rng, Rng};
use sharp_p2p_common::job::Job;
use std::{env, path::PathBuf};

pub struct TestFixture {
    job: Job,
    program_path: PathBuf,
}

fn fixture() -> TestFixture {
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
            hex::encode(rng.gen::<[u8; 32]>()).as_str(),
            libsecp256k1::SecretKey::random(&mut rng),
        ),
        program_path,
    }
}

#[tokio::test]
async fn run_single_job() {
    let fixture = fixture();

    let runner = CairoRunner::new(fixture.program_path);
    runner.run(fixture.job).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_jobs() {
    let fixture = fixture();

    let runner = CairoRunner::new(fixture.program_path);
    let job = runner.run(fixture.job).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}

#[tokio::test]
async fn run_multiple_jobs() {
    let fixture1 = fixture();
    let fixture2 = fixture();

    let runner = CairoRunner::new(fixture1.program_path);
    let mut futures = FuturesUnordered::new();

    let job1 = runner.run(fixture1.job).unwrap();
    let job2 = runner.run(fixture2.job).unwrap();

    futures.push(job1);
    futures.push(job2);

    while let Some(job_trace) = futures.next().await {
        job_trace.unwrap();
    }
}

#[tokio::test]
async fn abort_multiple_jobs() {
    let fixture1 = fixture();
    let fixture2 = fixture();

    let runner = CairoRunner::new(fixture1.program_path);
    let mut futures = FuturesUnordered::new();

    let job1 = runner.run(fixture1.job).unwrap();
    let job2 = runner.run(fixture2.job).unwrap();

    job1.abort().await.unwrap();
    job2.abort().await.unwrap();

    futures.push(job1);
    futures.push(job2);

    while let Some(job_trace) = futures.next().await {
        job_trace.unwrap_err();
    }
}
