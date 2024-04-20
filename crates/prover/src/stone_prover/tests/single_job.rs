use crate::{
    stone_prover::{tests::models::fixture, StoneProver},
    traits::ProverController,
};

#[tokio::test]
async fn run_single_job_trace() {
    let fixture = fixture();

    let prover = StoneProver::new(fixture.cpu_air_prover_config, fixture.cpu_air_params);
    prover.run(fixture.job_trace).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_job_trace() {
    let fixture = fixture();

    let prover = StoneProver::new(fixture.cpu_air_prover_config, fixture.cpu_air_params);
    let job = prover.run(fixture.job_trace).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}
