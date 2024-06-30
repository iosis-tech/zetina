use std::{collections::BTreeSet, hash::Hash};

use rand::Rng;

pub struct JobRecord<J> {
    ordered_set: BTreeSet<J>,
}

impl<J> JobRecord<J>
where
    J: Hash + Ord,
{
    pub fn new() -> Self {
        Self { ordered_set: BTreeSet::new() }
    }

    pub fn register_job(&mut self, job: J) {
        self.ordered_set.insert(job);
    }

    pub fn remove_job(&mut self, job: &J) -> bool {
        self.ordered_set.remove(job)
    }

    pub async fn take_job(&mut self) -> Option<J> {
        // add random wait to simulate network overhead
        let random = {
            let mut rng = rand::thread_rng();
            rng.gen_range(0..1000)
        };
        tokio::time::sleep(std::time::Duration::from_millis(random)).await;
        self.ordered_set.pop_last()
    }

    pub fn is_empty(&mut self) -> bool {
        self.ordered_set.is_empty()
    }
}

impl<J> Default for JobRecord<J>
where
    J: Hash + Ord,
{
    fn default() -> Self {
        Self::new()
    }
}
