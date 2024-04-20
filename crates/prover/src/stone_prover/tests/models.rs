use crate::stone_prover::types::{
    config::Config,
    params::{Fri, Params, Stark},
};
use sharp_p2p_common::job_trace::JobTrace;
use std::{env, fs, io::Write, path::PathBuf};
use tempfile::NamedTempFile;

pub struct TestFixture {
    pub job_trace: JobTrace,
    pub cpu_air_prover_config: Config,
    pub cpu_air_params: Params,
}

pub fn fixture() -> TestFixture {
    let ws_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"))
            .join("../../");
    let air_private_input_path = ws_root.join("crates/tests/cairo/air_private_input.json");
    let air_public_input_path = ws_root.join("crates/tests/cairo/air_public_input.json");
    let memory_path = ws_root.join("crates/tests/cairo/memory");
    let trace_path = ws_root.join("crates/tests/cairo/trace");

    let mut air_public_input = NamedTempFile::new().unwrap();
    air_public_input.write_all(&fs::read(air_public_input_path).unwrap()).unwrap();

    let mut air_private_input = NamedTempFile::new().unwrap();
    air_private_input.write_all(&fs::read(air_private_input_path).unwrap()).unwrap();

    let mut memory = NamedTempFile::new().unwrap();
    memory.write_all(&fs::read(memory_path).unwrap()).unwrap();

    let mut trace = NamedTempFile::new().unwrap();
    trace.write_all(&fs::read(trace_path).unwrap()).unwrap();

    TestFixture {
        job_trace: JobTrace { air_public_input, air_private_input, memory, trace },
        cpu_air_prover_config: Config::default(),
        cpu_air_params: Params {
            stark: Stark {
                fri: Fri {
                    fri_step_list: vec![0, 4, 4, 3],
                    last_layer_degree_bound: 128,
                    n_queries: 1,
                    proof_of_work_bits: 1,
                },
                log_n_cosets: 1,
            },
            ..Default::default()
        },
    }
}
