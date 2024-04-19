use crate::{
    errors::RunnerControllerError,
    traits::{Runner, RunnerController},
    types::{
        input::{BootloaderInput, Task},
        layout::Layout,
    },
};
use sharp_p2p_common::{hash, job::Job, job_trace::JobTrace};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};
use std::{env, io::Write, path::PathBuf};
use tempfile::NamedTempFile;
use tokio::process::{Child, Command};
use tracing::{debug, trace};

pub struct CairoRunner {
    tasks: HashMap<u64, Child>,
}

impl Runner for CairoRunner {
    fn init() -> impl RunnerController {
        Self { tasks: HashMap::new() }
    }
}

impl RunnerController for CairoRunner {
    async fn run(&mut self, job: Job) -> Result<JobTrace, RunnerControllerError> {
        let cargo_target_dir =
            PathBuf::from(env::var("CARGO_TARGET_DIR").expect("CARGO_TARGET_DIR env not present"));
        let bootloader_out_name = PathBuf::from(
            env::var("BOOTLOADER_OUT_NAME").expect("BOOTLOADER_OUT_NAME env not present"),
        );

        let program = cargo_target_dir.join(&bootloader_out_name);
        let layout: &str = Layout::RecursiveWithPoseidon.into();

        let mut cairo_pie = NamedTempFile::new()?;
        cairo_pie.write_all(&job.cairo_pie)?;

        let input = BootloaderInput {
            tasks: vec![Task { path: cairo_pie.path().to_path_buf(), ..Default::default() }],
            ..Default::default()
        };

        let mut program_input = NamedTempFile::new()?;
        program_input.write_all(&serde_json::to_string(&input)?.into_bytes())?;

        // outputs
        let air_public_input = NamedTempFile::new()?;
        let air_private_input = NamedTempFile::new()?;
        let trace = NamedTempFile::new()?;
        let memory = NamedTempFile::new()?;

        let task = Command::new("cairo-run")
            .arg("--program")
            .arg(program.as_path())
            .arg("--layout")
            .arg(layout)
            .arg("--program_input")
            .arg(program_input.path())
            .arg("--air_public_input")
            .arg(air_public_input.path())
            .arg("--air_private_input")
            .arg(air_private_input.path())
            .arg("--trace_file")
            .arg(trace.path())
            .arg("--memory_file")
            .arg(memory.path())
            .arg("--proof_mode")
            .arg("--print_output")
            .spawn()?;

        let job_hash = hash!(job);

        debug!("task {} spawned", job_hash);
        self.tasks.insert(job_hash.to_owned(), task);

        let task_status = self
            .tasks
            .get_mut(&job_hash)
            .ok_or(RunnerControllerError::TaskNotFound)?
            .wait()
            .await?;

        trace!("task {} woke up", job_hash);
        if !task_status.success() {
            debug!("task terminated {}", job_hash);
            return Err(RunnerControllerError::TaskTerminated);
        }

        let task_output = self
            .tasks
            .remove(&job_hash)
            .ok_or(RunnerControllerError::TaskNotFound)?
            .wait_with_output()
            .await?;
        trace!("task {} output {:?}", job_hash, task_output);

        Ok(JobTrace { air_public_input, air_private_input, memory, trace })
    }

    fn terminate(&mut self, job_hash: u64) -> Result<(), RunnerControllerError> {
        self.tasks.get_mut(&job_hash).ok_or(RunnerControllerError::TaskNotFound)?.start_kill()?;
        trace!("task scheduled for termination {}", job_hash);
        Ok(())
    }

    fn drop(mut self) -> Result<(), RunnerControllerError> {
        let keys: Vec<u64> = self.tasks.keys().cloned().collect();
        for job_hash in keys.iter() {
            self.tasks
                .get_mut(job_hash)
                .ok_or(RunnerControllerError::TaskNotFound)?
                .start_kill()?;
            trace!("task scheduled for termination {}", job_hash);
        }
        Ok(())
    }
}
