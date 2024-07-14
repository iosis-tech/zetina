use crate::{
    cairo_runner::{tests::models::fixture, CairoRunner},
    traits::RunnerController,
};
use starknet::signers::SigningKey;

#[tokio::test]
async fn run_single_job() {
    let fixture = fixture();
    let runner = CairoRunner::new(fixture.program_path, SigningKey::from_random().verifying_key());
    runner.run(fixture.job).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_jobs() {
    let fixture = fixture();
    let runner = CairoRunner::new(fixture.program_path, SigningKey::from_random().verifying_key());
    let job = runner.run(fixture.job).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}
