use sharp_p2p_common::job::Job;

use std::collections::VecDeque;

#[derive(Debug)]
pub struct Store {
    /// For delegator, FIFO queue to publish message
    /// For executor, FIFO queue to prove job
    pub job_queue: VecDeque<Job>,
}

impl Store {
    pub fn new() -> Self {
        Self { job_queue: VecDeque::new() }
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}
