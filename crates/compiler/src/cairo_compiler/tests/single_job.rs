use crate::{
    cairo_compiler::{tests::models::fixture, CairoCompiler},
    traits::CompilerController,
};

#[tokio::test]
async fn run_single_job() {
    let fixture = fixture();

    let compiler = CairoCompiler::new();
    compiler.run(fixture.program_path, fixture.program_input_path).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_jobs() {
    let fixture = fixture();

    let runner = CairoCompiler::new();
    let job = runner.run(fixture.program_path, fixture.program_input_path).unwrap();
    job.abort().await.unwrap();
    job.await.unwrap_err();
}
