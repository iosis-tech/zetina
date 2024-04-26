use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use libp2p::identity::ecdsa::Keypair;
use sharp_p2p_prover::{stone_prover::StoneProver, traits::ProverController};
use sharp_p2p_runner::{
    cairo_runner::{tests::models::fixture as runner_fixture, CairoRunner},
    traits::RunnerController,
};

#[tokio::test]
async fn run_single_job() {
    let runner_fixture = runner_fixture();

    let identity = Keypair::generate();
    let runner = CairoRunner::new(runner_fixture.program_path, identity.public());
    let prover = StoneProver::new();

    runner
        .run(runner_fixture.job)
        .unwrap()
        .map(|job_trace| prover.run(job_trace.unwrap()).unwrap())
        .flatten()
        .await
        .unwrap();
}

#[tokio::test]
async fn run_multiple_job() {
    let runner_fixture1 = runner_fixture();
    let runner_fixture2 = runner_fixture();

    let identity = Keypair::generate();
    let runner = CairoRunner::new(runner_fixture1.program_path, identity.public());
    let prover = StoneProver::new();
    let mut futures = FuturesUnordered::new();

    let job_witness1 = runner
        .run(runner_fixture1.job)
        .unwrap()
        .map(|job_trace| prover.run(job_trace.unwrap()).unwrap())
        .flatten()
        .boxed_local();
    let job_witness2 = runner
        .run(runner_fixture2.job)
        .unwrap()
        .map(|job_trace| prover.run(job_trace.unwrap()).unwrap())
        .flatten()
        .boxed_local();

    futures.push(job_witness1);
    futures.push(job_witness2);

    while let Some(job_witness) = futures.next().await {
        job_witness.unwrap();
    }
}
