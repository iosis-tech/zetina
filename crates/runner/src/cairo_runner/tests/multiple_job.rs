use crate::{
    cairo_runner::{tests::models::fixture, CairoRunner},
    traits::RunnerController,
};
use futures::{stream::FuturesUnordered, StreamExt};
use starknet::signers::SigningKey;

#[tokio::test]
async fn run_multiple_jobs() {
    let fixture1 = fixture();
    let fixture2 = fixture();

    let runner = CairoRunner::new(fixture1.program_path, SigningKey::from_random().verifying_key());
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

    let runner = CairoRunner::new(fixture1.program_path, SigningKey::from_random().verifying_key());
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
