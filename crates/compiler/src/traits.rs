use std::path::PathBuf;

use crate::errors::CompilerControllerError;
use sharp_p2p_common::process::Process;

pub trait CompilerController {
    type ProcessResult;
    fn run(
        &self,
        program_path: PathBuf,
        program_input_path: PathBuf,
    ) -> Result<Process<Self::ProcessResult>, CompilerControllerError>;
}
