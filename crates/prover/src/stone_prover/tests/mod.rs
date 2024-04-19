use std::{env, fs, io::Write, path::PathBuf};

use sharp_p2p_common::job_trace::JobTrace;
use tempfile::NamedTempFile;

use crate::{stone_prover::StoneProver, traits::ProverController};

// use super::types::{
//     config::{Config, Fri, Stark},
//     params::Params,
// };

fn job_trace() -> JobTrace {
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let air_private_input_path =
        package_root.join("src/stone_prover/tests/cairo/air_private_input.json");
    let air_public_input_path =
        package_root.join("src/stone_prover/tests/cairo/air_public_input.json");
    let memory_path = package_root.join("src/stone_prover/tests/cairo/memory");
    let trace_path = package_root.join("src/stone_prover/tests/cairo/trace");

    let mut air_public_input = NamedTempFile::new().unwrap();
    air_public_input.write_all(&fs::read(air_public_input_path).unwrap()).unwrap();

    let mut air_private_input = NamedTempFile::new().unwrap();
    air_private_input.write_all(&fs::read(air_private_input_path).unwrap()).unwrap();

    let mut memory = NamedTempFile::new().unwrap();
    memory.write_all(&fs::read(memory_path).unwrap()).unwrap();

    let mut trace = NamedTempFile::new().unwrap();
    trace.write_all(&fs::read(trace_path).unwrap()).unwrap();

    JobTrace { air_public_input, air_private_input, memory, trace }
}

// fn config() -> (Config, Params) {
//     let config = Config {
//         stark: Stark {
//             fri: Fri {
//                 fri_step_list: vec![0, 4, 4, 3],
//                 last_layer_degree_bound: 128,
//                 n_queries: 10,
//                 proof_of_work_bits: 30,
//             },
//             log_n_cosets: 2,
//         },
//         ..Default::default()
//     };
//     let params = Params::default();
//     (config, params)
// }

#[tokio::test]
async fn run_single_job_trace() {
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let cpu_air_prover_config_path =
        package_root.join("src/stone_prover/tests/cairo/cpu_air_prover_config.json");
    let cpu_air_params_path = package_root.join("src/stone_prover/tests/cairo/cpu_air_params.json");

    let mut cpu_air_prover_config = NamedTempFile::new().unwrap();
    cpu_air_prover_config.write_all(&fs::read(cpu_air_prover_config_path).unwrap()).unwrap();

    let mut cpu_air_params = NamedTempFile::new().unwrap();
    cpu_air_params.write_all(&fs::read(cpu_air_params_path).unwrap()).unwrap();

    let prover = StoneProver::new(cpu_air_prover_config, cpu_air_params);
    prover.run(job_trace()).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_single_job_trace() {
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let cpu_air_prover_config_path =
        package_root.join("src/stone_prover/tests/cairo/cpu_air_prover_config.json");
    let cpu_air_params_path = package_root.join("src/stone_prover/tests/cairo/cpu_air_params.json");

    let mut cpu_air_prover_config = NamedTempFile::new().unwrap();
    cpu_air_prover_config.write_all(&fs::read(cpu_air_prover_config_path).unwrap()).unwrap();

    let mut cpu_air_params = NamedTempFile::new().unwrap();
    cpu_air_params.write_all(&fs::read(cpu_air_params_path).unwrap()).unwrap();

    let prover = StoneProver::new(cpu_air_prover_config, cpu_air_params);
    let job = prover.run(job_trace()).unwrap();
    job.abort().await.unwrap();

    job.await.unwrap_err();
}

#[tokio::test]
async fn run_multiple_job_traces() {
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let cpu_air_prover_config_path =
        package_root.join("src/stone_prover/tests/cairo/cpu_air_prover_config.json");
    let cpu_air_params_path = package_root.join("src/stone_prover/tests/cairo/cpu_air_params.json");

    let mut cpu_air_prover_config = NamedTempFile::new().unwrap();
    cpu_air_prover_config.write_all(&fs::read(cpu_air_prover_config_path).unwrap()).unwrap();

    let mut cpu_air_params = NamedTempFile::new().unwrap();
    cpu_air_params.write_all(&fs::read(cpu_air_params_path).unwrap()).unwrap();

    let prover = StoneProver::new(cpu_air_prover_config, cpu_air_params);
    prover.run(job_trace()).unwrap().await.unwrap();
    prover.run(job_trace()).unwrap().await.unwrap();
}

#[tokio::test]
async fn abort_multiple_job_traces() {
    let package_root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR env not present"));
    let cpu_air_prover_config_path =
        package_root.join("src/stone_prover/tests/cairo/cpu_air_prover_config.json");
    let cpu_air_params_path = package_root.join("src/stone_prover/tests/cairo/cpu_air_params.json");

    let mut cpu_air_prover_config = NamedTempFile::new().unwrap();
    cpu_air_prover_config.write_all(&fs::read(cpu_air_prover_config_path).unwrap()).unwrap();

    let mut cpu_air_params = NamedTempFile::new().unwrap();
    cpu_air_params.write_all(&fs::read(cpu_air_params_path).unwrap()).unwrap();

    let prover = StoneProver::new(cpu_air_prover_config, cpu_air_params);
    let job1 = prover.run(job_trace()).unwrap();
    let job2 = prover.run(job_trace()).unwrap();

    job1.abort().await.unwrap();
    job2.abort().await.unwrap();

    job1.await.unwrap_err();
    job2.await.unwrap_err();
}
