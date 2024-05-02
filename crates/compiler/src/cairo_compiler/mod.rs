use crate::{errors::CompilerControllerError, traits::CompilerController};
use async_process::Stdio;
use futures::Future;
use rand::{thread_rng, Rng};
use serde_json::json;
use sharp_p2p_common::layout::Layout;
use sharp_p2p_common::process::Process;
use std::io::Write;
use std::path::PathBuf;
use std::{io::Read, pin::Pin};
use tempfile::NamedTempFile;
use tokio::{process::Command, select, sync::mpsc};
use tracing::debug;

pub mod tests;

pub struct CairoCompiler {}

impl CairoCompiler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CairoCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerController for CairoCompiler {
    fn run(
        &self,
        program_path: PathBuf,
        _program_input_path: PathBuf,
    ) -> Result<Process<Result<Vec<u8>, CompilerControllerError>>, CompilerControllerError> {
        let (terminate_tx, mut terminate_rx) = mpsc::channel::<()>(10);
        let future: Pin<Box<dyn Future<Output = Result<Vec<u8>, CompilerControllerError>> + '_>> =
            Box::pin(async move {
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

                // TODO remove it is just to make every job a little diffirent for testing purposes
                let mut random_input = NamedTempFile::new()?;
                let mut rng = thread_rng();
                random_input.write_all(
                    json!({
                        "fibonacci_claim_index": rng.gen_range(10..10000)
                    })
                    .to_string()
                    .as_bytes(),
                )?;

                // output
                let mut cairo_pie = NamedTempFile::new()?;

                let mut task = Command::new("cairo-run")
                    .arg("--program")
                    .arg(output.path())
                    .arg("--layout")
                    .arg(layout)
                    .arg("--program_input")
                    .arg(random_input.path())
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

                // cairo run had finished
                let mut cairo_pie_compressed = Vec::new();
                cairo_pie.read_to_end(&mut cairo_pie_compressed)?;

                Ok(cairo_pie_compressed)
            });

        Ok(Process::new(future, terminate_tx))
    }
}
