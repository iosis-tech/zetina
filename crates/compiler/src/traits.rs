use crate::errors::CompilerControllerError;
use sharp_p2p_common::{job::Job, process::Process};
use std::path::PathBuf;

/*
    The `CompilerController` trait is responsible for taking a user's program and preparing a `Job` object.
    This process involves compiling the user's code and creating a Cairo PIE (Proof-of-Inclusion-Execution) object from it.
    The resulting `Job` object encapsulates the necessary information for later execution by a `RunnerController`.
    The `run` method accepts the paths to the program and its input, returning a `Result` containing a `Process` object.
    Upon successful completion, it yields a `Job` object, ready to be utilized by a `RunnerController` to execute the program.
*/

pub trait CompilerController {
    fn run(
        &self,
        program_path: PathBuf,
        program_input_path: PathBuf,
    ) -> Result<Process<Result<Job, CompilerControllerError>>, CompilerControllerError>;
}
