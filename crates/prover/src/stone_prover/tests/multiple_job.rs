use crate::{
    stone_prover::{tests::models::fixture, StoneProver},
    traits::ProverController,
};
use futures::{stream::FuturesUnordered, StreamExt};

#[tokio::test]
async fn run_multiple_job_traces() {
    let fixture1 = fixture();
    let fixture2 = fixture();

    let prover = StoneProver::new(fixture1.cpu_air_prover_config, fixture1.cpu_air_params);
    let mut futures = FuturesUnordered::new();

    let job1 = prover.run(fixture1.job_trace).unwrap();
    let job2 = prover.run(fixture2.job_trace).unwrap();

    futures.push(job1);
    futures.push(job2);

    while let Some(job_trace) = futures.next().await {
        job_trace.unwrap();
    }
}

#[tokio::test]
async fn abort_multiple_job_traces() {
    let fixture1 = fixture();
    let fixture2 = fixture();

    let prover = StoneProver::new(fixture1.cpu_air_prover_config, fixture1.cpu_air_params);
    let mut futures = FuturesUnordered::new();

    let job1 = prover.run(fixture1.job_trace).unwrap();
    let job2 = prover.run(fixture2.job_trace).unwrap();

    job1.abort().await.unwrap();
    job2.abort().await.unwrap();

    futures.push(job1);
    futures.push(job2);

    while let Some(job_trace) = futures.next().await {
        job_trace.unwrap_err();
    }
}
