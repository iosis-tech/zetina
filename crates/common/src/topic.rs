use libp2p::gossipsub::IdentTopic;

use crate::network::Network;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    NewJob,
    PickedJob,
}

impl Topic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Topic::NewJob => "new-job",
            Topic::PickedJob => "picked-job",
        }
    }
}

pub fn gossipsub_ident_topic(network: Network, topic: Topic) -> IdentTopic {
    let network = network.as_str();
    let topic = topic.as_str();
    let s = format!("/{network}/{topic}");
    IdentTopic::new(s)
}
