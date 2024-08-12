use libp2p::{kad, PeerId};
use std::{collections::BTreeMap, future::Future, pin::Pin, time::Duration};
use tokio::{sync::mpsc, time::sleep};
use zetina_common::process::Process;

pub struct BidQueue {}

impl BidQueue {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BidQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl BidQueue {
    pub fn run<'future>(
        job_hash: kad::RecordKey,
    ) -> (
        Process<'future, Result<(kad::RecordKey, BTreeMap<u64, Vec<PeerId>>), BidControllerError>>,
        mpsc::Sender<(u64, PeerId)>,
    ) {
        let (terminate_tx, mut terminate_rx) = mpsc::channel::<()>(10);
        let (bid_tx, mut bid_rx) = mpsc::channel::<(u64, PeerId)>(10);
        let future: Pin<
            Box<
                dyn Future<
                        Output = Result<
                            (kad::RecordKey, BTreeMap<u64, Vec<PeerId>>),
                            BidControllerError,
                        >,
                    > + Send
                    + '_,
            >,
        > = Box::pin(async move {
            let duration = Duration::from_secs(5);
            let mut bids: Option<BTreeMap<u64, Vec<PeerId>>> = Some(BTreeMap::new());
            loop {
                tokio::select! {
                    Some((price, peerid)) = bid_rx.recv() => {
                        match &mut bids {
                            Some(bids) => {
                                match bids.get_mut(&price) {
                                    Some(vec) => {
                                        vec.push(peerid);
                                    },
                                    None => {
                                        bids.insert(price, vec![peerid]);
                                    }
                                }
                            },
                            None => break Err(BidControllerError::BidsTerminated)
                        }
                    }
                    _ = sleep(duration) => {
                        break Ok((job_hash, bids.take().ok_or(BidControllerError::BidsTerminated)?))
                    }
                    _ = terminate_rx.recv() => {
                        break Err(BidControllerError::TaskTerminated);
                    }
                    else => break Err(BidControllerError::TaskTerminated)
                }
            }
        });

        (Process::new(future, terminate_tx), bid_tx)
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BidControllerError {
    #[error("task not found")]
    TaskTerminated,

    #[error("task not found")]
    BidsTerminated,

    #[error("task not found")]
    NoBid,
}
