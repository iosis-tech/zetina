use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use libsecp256k1::{PublicKey, SecretKey};
use rand::thread_rng;
use sharp_p2p_prover::{
    stone_prover::{
        types::{
            config::Config,
            params::{Fri, Params, Stark},
        },
        StoneProver,
    },
    traits::ProverController,
};
use sharp_p2p_runner::{
    cairo_runner::{tests::models::fixture as runner_fixture, CairoRunner},
    traits::RunnerController,
};

pub fn config() -> Config {
    Config::default()
}

pub fn params() -> Params {
    Params {
        stark: Stark {
            fri: Fri {
                fri_step_list: vec![0, 4, 4, 4, 1],
                last_layer_degree_bound: 128,
                n_queries: 1,
                proof_of_work_bits: 1,
            },
            log_n_cosets: 1,
        },
        ..Default::default()
    }
}

#[tokio::test]
async fn run_single_job() {
    let mut rng = thread_rng();
    let runner_fixture = runner_fixture();

    let runner = CairoRunner::new(
        runner_fixture.program_path,
        PublicKey::from_secret_key(&SecretKey::random(&mut rng)),
    );
    let prover = StoneProver::new(config(), params());

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
    let mut rng = thread_rng();
    let runner_fixture1 = runner_fixture();
    let runner_fixture2 = runner_fixture();

    let runner = CairoRunner::new(
        runner_fixture1.program_path,
        PublicKey::from_secret_key(&SecretKey::random(&mut rng)),
    );
    let prover = StoneProver::new(config(), params());
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
