use libp2p::gossipsub::IdentTopic;

use crate::network::Network;

/*
    Topic
    This defines the topic of the message used in the gossipsub protocol.
    The topic is used to filter messages and route them to the correct subscribers.
    `NewJob` is used to notify the network of a new job from Delegator to Executor.
    `PickedJob` is used to notify the network of a job that has been picked up by an Executor.
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    NewJob,
    PickedJob,
    FinishedJob,
}

impl Topic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Topic::NewJob => "new-job",
            Topic::PickedJob => "picked-job",
            Topic::FinishedJob => "finished-job",
        }
    }
}

pub fn gossipsub_ident_topic(network: Network, topic: Topic) -> IdentTopic {
    let network = network.as_str();
    let topic = topic.as_str();
    let s = format!("/{network}/{topic}");
    IdentTopic::new(s)
}
