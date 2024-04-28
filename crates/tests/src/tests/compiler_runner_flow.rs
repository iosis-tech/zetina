use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use sharp_p2p_compiler::cairo_compiler::tests::models::fixture as compiler_fixture;
use sharp_p2p_compiler::cairo_compiler::CairoCompiler;
use sharp_p2p_compiler::traits::CompilerController;
use sharp_p2p_runner::cairo_runner::tests::models::fixture as runner_fixture;
use sharp_p2p_runner::cairo_runner::CairoRunner;
use sharp_p2p_runner::traits::RunnerController;
use starknet::signers::SigningKey;
use starknet_crypto::FieldElement;

#[tokio::test]
async fn run_single_job() {
    let compiler_fixture = compiler_fixture();
    let runner_fixture = runner_fixture();

    let compiler_identity = SigningKey::from_random();
    let compiler = CairoCompiler::new(&compiler_identity, FieldElement::ZERO);
    let runner_identity = SigningKey::from_random().verifying_key();
    let runner = CairoRunner::new(runner_fixture.program_path, &runner_identity);

    compiler
        .run(compiler_fixture.program_path, compiler_fixture.program_input_path)
        .unwrap()
        .map(|job| {
            println!("job: {:?}", job);
            runner.run(job.unwrap()).unwrap()
        })
        .flatten()
        .await
        .unwrap();
}

#[tokio::test]
async fn run_multiple_job() {
    let compiler_fixture1 = compiler_fixture();
    let compiler_fixture2 = compiler_fixture();
    let runner_fixture1 = runner_fixture();

    let compiler_identity = SigningKey::from_random();
    let compiler = CairoCompiler::new(&compiler_identity, FieldElement::ZERO);
    let runner_identity = SigningKey::from_random().verifying_key();
    let runner = CairoRunner::new(runner_fixture1.program_path, &runner_identity);

    let mut futures = FuturesUnordered::new();

    let job_trace1 = compiler
        .run(compiler_fixture1.program_path, compiler_fixture1.program_input_path)
        .unwrap()
        .map(|job| runner.run(job.unwrap()).unwrap())
        .flatten()
        .boxed_local();

    let job_trace2 = compiler
        .run(compiler_fixture2.program_path, compiler_fixture2.program_input_path)
        .unwrap()
        .map(|job| runner.run(job.unwrap()).unwrap())
        .flatten()
        .boxed_local();

    futures.push(job_trace1);
    futures.push(job_trace2);

    while let Some(job_trace) = futures.next().await {
        job_trace.unwrap();
    }
}
