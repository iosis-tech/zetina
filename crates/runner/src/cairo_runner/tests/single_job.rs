use crate::{
    cairo_runner::{tests::models::fixture, CairoRunner},
    traits::RunnerController,
};
use libsecp256k1::{PublicKey, SecretKey};
use rand::thread_rng;

#[tokio::test]
async fn run_single_job() {
    let mut rng = thread_rng();
    let fixture = fixture();

    let runner = CairoRunner::new(
        fixture.program_path,
        PublicKey::from_secret_key(&SecretKey::random(&mut rng)),
    );
    runner.run(fixture.job).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_jobs() {
    let mut rng = thread_rng();
    let fixture = fixture();

    let runner = CairoRunner::new(
        fixture.program_path,
        PublicKey::from_secret_key(&SecretKey::random(&mut rng)),
    );
    let job = runner.run(fixture.job).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}
