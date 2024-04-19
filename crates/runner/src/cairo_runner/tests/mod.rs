use super::CairoRunner;
use crate::traits::RunnerController;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use sharp_p2p_common::job::Job;
use std::{env, path::PathBuf};

fn random_job(cairo_pie_path: PathBuf, rng: &mut ThreadRng) -> Job {
    Job::new(
        rng.gen(),
        rng.gen(),
        cairo_pie_path,
        hex::encode(rng.gen::<[u8; 32]>()).as_str(),
        libsecp256k1::SecretKey::random(rng),
    )
}

#[tokio::test]
async fn run_single_job() {
    let mut rng = thread_rng();
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let cairo_pie_path = package_root.join("src/cairo_runner/tests/cairo/fibonacci_pie.zip");

    let runner = CairoRunner::new(package_root.join("../../target/bootloader.json"));

    runner.run(random_job(cairo_pie_path.to_owned(), &mut rng)).unwrap().await.unwrap();
}

#[tokio::test]
async fn run_multiple_jobs() {
    let mut rng = thread_rng();
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let cairo_pie_path = package_root.join("src/cairo_runner/tests/cairo/fibonacci_pie.zip");

    let runner = CairoRunner::new(package_root.join("../../target/bootloader.json"));
    let mut futures = FuturesUnordered::new();

    let job1 = runner.run(random_job(cairo_pie_path.to_owned(), &mut rng)).unwrap();
    let job2 = runner.run(random_job(cairo_pie_path.to_owned(), &mut rng)).unwrap();

    futures.push(job1);
    futures.push(job2);

    while let Some(job_trace) = futures.next().await {
        job_trace.unwrap();
    }
}

#[tokio::test]
async fn abort_single_jobs() {
    let mut rng = thread_rng();
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let cairo_pie_path = package_root.join("src/cairo_runner/tests/cairo/fibonacci_pie.zip");

    let runner = CairoRunner::new(package_root.join("../../target/bootloader.json"));

    let job = runner.run(random_job(cairo_pie_path.to_owned(), &mut rng)).unwrap();
    job.abort().await.unwrap();

    job.await.unwrap_err();
}

#[tokio::test]
async fn abort_multiple_jobs() {
    let mut rng = thread_rng();
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let cairo_pie_path = package_root.join("src/cairo_runner/tests/cairo/fibonacci_pie.zip");

    let runner = CairoRunner::new(package_root.join("../../target/bootloader.json"));
    let mut futures = FuturesUnordered::new();

    let job1 = runner.run(random_job(cairo_pie_path.to_owned(), &mut rng)).unwrap();
    let job2 = runner.run(random_job(cairo_pie_path.to_owned(), &mut rng)).unwrap();

    job1.abort().await.unwrap();
    job2.abort().await.unwrap();

    futures.push(job1);
    futures.push(job2);

    while let Some(job_trace) = futures.next().await {
        job_trace.unwrap_err();
    }
}
