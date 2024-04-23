use rand::thread_rng;

use crate::{
    cairo_compiler::{tests::models::fixture, CairoCompiler},
    traits::CompilerController,
};

#[tokio::test]
async fn run_single_job() {
    let mut rng = thread_rng();
    let fixture = fixture();

    let compiler = CairoCompiler::new(libsecp256k1::SecretKey::random(&mut rng));
    compiler.run(fixture.program_path, fixture.program_input_path).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_jobs() {
    let mut rng = thread_rng();
    let fixture = fixture();

    let compiler = CairoCompiler::new(libsecp256k1::SecretKey::random(&mut rng));
    let job = compiler.run(fixture.program_path, fixture.program_input_path).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}
