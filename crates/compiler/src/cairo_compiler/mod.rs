use crate::{errors::CompilerControllerError, traits::CompilerController};
use async_process::Stdio;
use futures::Future;
use sharp_p2p_common::layout::Layout;
use sharp_p2p_common::{job::Job, process::Process};
use std::path::PathBuf;
use std::{io::Read, pin::Pin};
use tempfile::NamedTempFile;
use tokio::{process::Command, select, sync::mpsc};
use tracing::debug;

pub struct CairoCompiler {
    program_path: PathBuf,
}

impl CairoCompiler {
    pub fn new(program_path: PathBuf) -> Self {
        Self { program_path }
    }
}

impl CompilerController for CairoCompiler {
    type ProcessResult = Result<Job, CompilerControllerError>;
    fn run(
        &self,
        program_path: PathBuf,
        program_input_path: PathBuf,
    ) -> Result<Process<Self::ProcessResult>, CompilerControllerError> {
        let (terminate_tx, mut terminate_rx) = mpsc::channel::<()>(10);
        let future: Pin<Box<dyn Future<Output = Self::ProcessResult> + '_>> =
            Box::pin(async move {
                let layout: &str = Layout::RecursiveWithPoseidon.into();

                // output
                let mut cairo_pie = NamedTempFile::new()?;

                let mut task = Command::new("cairo-run")
                    .arg("--program")
                    .arg(program_path.as_path())
                    .arg("--layout")
                    .arg(layout)
                    .arg("--program_input")
                    .arg(program_input_path.as_path())
                    .arg("--cairo_pie_output")
                    .arg(cairo_pie.path())
                    .arg("--print_output")
                    .stdout(Stdio::null())
                    .spawn()?;

                debug!("program {:?} is spawn", program_path);

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
                let mut cairo_pie_bytes = Vec::new();
                cairo_pie.read_to_end(&mut cairo_pie_bytes)?;

                Ok(Job { cairo_pie: cairo_pie_bytes, ..Default::default() })
            });

        Ok(Process::new(future, terminate_tx))
    }
}