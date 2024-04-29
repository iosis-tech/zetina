use crate::{
    cairo_compiler::{tests::models::fixture, CairoCompiler},
    traits::CompilerController,
};
use starknet::signers::SigningKey;
use starknet_crypto::FieldElement;

#[tokio::test]
async fn run_single_job() {
    let fixture = fixture();
    let identity = SigningKey::from_random();
    let compiler = CairoCompiler::new(&identity, FieldElement::ZERO);
    compiler.run(fixture.program_path, fixture.program_input_path).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_jobs() {
    let fixture = fixture();
    let identity = SigningKey::from_random();
    let compiler = CairoCompiler::new(&identity, FieldElement::ZERO);
    let job = compiler.run(fixture.program_path, fixture.program_input_path).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}
