use std::{env, path::PathBuf};

pub struct TestFixture {
    pub program_input_path: PathBuf,
    pub program_path: PathBuf,
}

pub fn fixture() -> TestFixture {
    let ws_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"))
            .join("../../");
    let program_path = ws_root.join("crates/tests/cairo/fibonacci.cairo");
    let program_input_path = ws_root.join("crates/tests/cairo/fibonacci_input.json");

    TestFixture { program_path, program_input_path }
}
