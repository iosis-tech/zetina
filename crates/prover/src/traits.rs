use crate::errors::ProverControllerError;
use sharp_p2p_common::{job_trace::JobTrace, process::Process};

pub trait ProverController {
    type ProcessResult;
    fn run(
        &self,
        job_trace: JobTrace,
    ) -> Result<Process<Self::ProcessResult>, ProverControllerError>;
}
