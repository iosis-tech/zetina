use starknet_crypto::FieldElement;

use crate::{
    cairo_compiler::{tests::models::fixture, CairoCompiler},
    traits::CompilerController,
};

#[tokio::test]
async fn run_single_job() {
    let fixture = fixture();
    let compiler =
        CairoCompiler::new(libp2p::identity::ecdsa::Keypair::generate(), FieldElement::ZERO);
    compiler.run(fixture.program_path, fixture.program_input_path).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_jobs() {
    let fixture = fixture();
    let compiler =
        CairoCompiler::new(libp2p::identity::ecdsa::Keypair::generate(), FieldElement::ZERO);
    let job = compiler.run(fixture.program_path, fixture.program_input_path).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}
