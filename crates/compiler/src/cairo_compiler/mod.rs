use crate::{errors::CompilerControllerError, traits::CompilerController};
use async_process::Stdio;
use futures::Future;
use starknet::signers::SigningKey;
use std::path::PathBuf;
use std::{io::Read, pin::Pin};
use tempfile::NamedTempFile;
use tokio::{process::Command, select, sync::mpsc};
use tracing::debug;
use zetina_common::job::JobData;
use zetina_common::layout::Layout;
use zetina_common::{job::Job, process::Process};

pub mod tests;

pub struct CairoCompiler<'identity> {
    signing_key: &'identity SigningKey,
}

impl<'identity> CairoCompiler<'identity> {
    pub fn new(signing_key: &'identity SigningKey) -> Self {
        Self { signing_key }
    }
}

impl<'identity> CompilerController for CairoCompiler<'identity> {
    fn run(
        &self,
        program_path: PathBuf,
        program_input_path: PathBuf,
    ) -> Result<Process<Result<Job, CompilerControllerError>>, CompilerControllerError> {
        let (terminate_tx, mut terminate_rx) = mpsc::channel::<()>(10);
        let future: Pin<
            Box<dyn Future<Output = Result<Job, CompilerControllerError>> + Send + '_>,
        > = Box::pin(async move {
            let layout: &str = Layout::RecursiveWithPoseidon.into();

            let output = NamedTempFile::new()?;

            let mut task = Command::new("cairo-compile")
                .arg(program_path.as_path())
                .arg("--output")
                .arg(output.path())
                .arg("--proof_mode")
                .stdout(Stdio::null())
                .spawn()?;

            debug!("program {:?} is compiling... ", program_path);

            loop {
                select! {
                    output = task.wait() => {
                        debug!("{:?}", output);
                        if !output?.success() {
                            return Err(CompilerControllerError::TaskTerminated);
                        }
                        let output = task.wait_with_output().await?;
                        debug!("{:?}", output);
                        break;
                    }
                    Some(()) = terminate_rx.recv() => {
                        task.start_kill()?;
                    }
                }
            }

            let mut cairo_pie = NamedTempFile::new()?;

            let mut task = Command::new("cairo-run")
                .arg("--program")
                .arg(output.path())
                .arg("--layout")
                .arg(layout)
                .arg("--program_input")
                .arg(program_input_path)
                .arg("--cairo_pie_output")
                .arg(cairo_pie.path())
                .arg("--print_output")
                .stdout(Stdio::null())
                .spawn()?;

            debug!("program {:?} is generating PIE... ", program_path);

            loop {
                select! {
                    output = task.wait() => {
                        debug!("{:?}", output);
                        if !output?.success() {
                            return Err(CompilerControllerError::TaskTerminated);
                        }
                        let output = task.wait_with_output().await?;
                        debug!("{:?}", output);
                        break;
                    }
                    Some(()) = terminate_rx.recv() => {
                        task.start_kill()?;
                    }
                }
            }

            let mut cairo_pie_compressed = Vec::new();
            cairo_pie.read_to_end(&mut cairo_pie_compressed)?;

            Ok(Job::try_from_job_data(JobData::new(0, cairo_pie_compressed), self.signing_key))
        });

        Ok(Process::new(future, terminate_tx))
    }
}
