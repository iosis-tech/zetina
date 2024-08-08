use libp2p::PeerId;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use zetina_common::{
    hash,
    job::{Job, JobBid},
};

pub struct JobBidQueue {
    map: HashMap<u64, (Job, HashMap<PeerId, u64>)>,
}

impl Default for JobBidQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl JobBidQueue {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn insert_job(&mut self, job: Job) -> Option<(Job, HashMap<PeerId, u64>)> {
        self.map.insert(hash!(job), (job, HashMap::new()))
    }

    pub fn get_best(&self, job_hash: u64) -> Option<(Job, PeerId, u64)> {
        self.map.get(&job_hash).and_then(|(job, bids)| {
            bids.iter()
                .min_by_key(|&(_, price)| price)
                .map(|(peer_id, &price)| (job.clone(), *peer_id, price))
        })
    }

    pub fn insert_bid(&mut self, job_bid: JobBid, identity: PeerId) {
        if let Some((_, map)) = self.map.get_mut(&job_bid.job_hash) {
            map.insert(identity, job_bid.price);
        }
    }

    pub fn remove_job(&mut self, job_hash: u64) -> Option<(Job, HashMap<PeerId, u64>)> {
        self.map.remove(&job_hash)
    }
}
