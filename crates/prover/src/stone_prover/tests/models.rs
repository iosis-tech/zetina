use sharp_p2p_common::job_trace::JobTrace;
use std::{env, fs, io::Write, path::PathBuf};
use tempfile::NamedTempFile;

pub struct TestFixture {
    pub job_trace: JobTrace,
}

pub fn fixture() -> TestFixture {
    let ws_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"))
            .join("../../");
    let air_private_input_path =
        ws_root.join("crates/tests/cairo/air_bootloader_private_input.json");
    let air_public_input_path = ws_root.join("crates/tests/cairo/air_bootloader_public_input.json");
    let memory_path = ws_root.join("crates/tests/cairo/bootloader.memory");
    let trace_path = ws_root.join("crates/tests/cairo/bootloader.trace");

    let mut air_public_input = NamedTempFile::new().unwrap();
    air_public_input.write_all(&fs::read(air_public_input_path).unwrap()).unwrap();

    let mut air_private_input = NamedTempFile::new().unwrap();
    air_private_input.write_all(&fs::read(air_private_input_path).unwrap()).unwrap();

    let mut memory = NamedTempFile::new().unwrap();
    memory.write_all(&fs::read(memory_path).unwrap()).unwrap();

    let mut trace = NamedTempFile::new().unwrap();
    trace.write_all(&fs::read(trace_path).unwrap()).unwrap();

    TestFixture { job_trace: JobTrace { air_public_input, air_private_input, memory, trace } }
}
