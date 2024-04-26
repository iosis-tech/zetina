use std::{collections::BTreeSet, hash::Hash};

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
        self.ordered_set.pop_last()
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
