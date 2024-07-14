use crate::{
    cairo_compiler::{tests::models::fixture, CairoCompiler},
    traits::CompilerController,
};
use futures::{stream::FuturesUnordered, StreamExt};
use starknet::signers::SigningKey;

#[tokio::test]
async fn run_multiple_jobs() {
    let fixture1 = fixture();
    let fixture2 = fixture();

    let identity = SigningKey::from_random();
    let compiler = CairoCompiler::new(&identity);
    let mut futures = FuturesUnordered::new();

    let job1 = compiler.run(fixture1.program_path, fixture1.program_input_path).unwrap();
    let job2 = compiler.run(fixture2.program_path, fixture2.program_input_path).unwrap();

    futures.push(job1);
    futures.push(job2);

    while let Some(job) = futures.next().await {
        job.unwrap();
    }
}

#[tokio::test]
async fn abort_multiple_jobs() {
    let fixture1 = fixture();
    let fixture2 = fixture();

    let identity = SigningKey::from_random();
    let compiler = CairoCompiler::new(&identity);
    let mut futures = FuturesUnordered::new();

    let job1 = runner.run(fixture1.program_path, fixture1.program_input_path).unwrap();
    let job2 = runner.run(fixture2.program_path, fixture2.program_input_path).unwrap();

    job1.abort().await.unwrap();
    job2.abort().await.unwrap();

    futures.push(job1);
    futures.push(job2);

    while let Some(job_trace) = futures.next().await {
        job_trace.unwrap_err();
    }
}
