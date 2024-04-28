use crate::{
    cairo_runner::{tests::models::fixture, CairoRunner},
    traits::RunnerController,
};
use libp2p::identity::ecdsa::Keypair;

#[tokio::test]
async fn run_single_job() {
    let fixture = fixture();
    let identity = Keypair::generate();
    let runner = CairoRunner::new(fixture.program_path, identity.public());
    runner.run(fixture.job).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_jobs() {
    let fixture = fixture();
    let identity = Keypair::generate();
    let runner = CairoRunner::new(fixture.program_path, identity.public());
    let job = runner.run(fixture.job).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}
