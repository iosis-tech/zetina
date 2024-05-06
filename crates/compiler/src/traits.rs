use crate::errors::CompilerControllerError;
use sharp_p2p_common::process::Process;
use std::path::PathBuf;

/*
    The `CompilerController` trait is responsible for taking a user's program and preparing a `Job` object.
    This process involves compiling the user's code and creating a Cairo PIE (Proof-of-Inclusion-Execution) object from it.
    The resulting `Vec<u8>` object that represents the Cairo PIE is then compressed and stored in the `JobData` object.
    Later, the `Job` object is then signed by the delegator's private key and sent to the network for execution.
*/

pub trait CompilerController {
    fn run(
        &self,
        program_path: PathBuf,
        program_input_path: PathBuf,
    ) -> Result<Process<Result<Vec<u8>, CompilerControllerError>>, CompilerControllerError>;
}
