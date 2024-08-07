use async_stream::stream;
use futures::stream::Stream;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic, TopicHash};
use libp2p::identity::Keypair;
use libp2p::swarm::{DialError, NetworkBehaviour, SwarmEvent};
use libp2p::{noise, tcp, yamux, Multiaddr, Swarm, SwarmBuilder};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use zetina_common::graceful_shutdown::shutdown_signal;
use zetina_common::job::{Job, JobBid, JobDelegation};
use zetina_common::job_witness::JobWitness;

#[derive(NetworkBehaviour)]
pub struct PeerBehaviour {
    gossipsub: gossipsub::Behaviour,
}

pub struct SwarmRunner {
    pub swarm: Swarm<PeerBehaviour>,
    pub p2p_multiaddr: Multiaddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Networking,
    Market,
    Delegation,
}

impl Topic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Topic::Networking => "networking",
            Topic::Market => "market",
            Topic::Delegation => "delegation",
        }
    }
}

impl From<Topic> for TopicHash {
    fn from(value: Topic) -> Self {
        IdentTopic::from(value).into()
    }
}

impl From<Topic> for IdentTopic {
    fn from(value: Topic) -> Self {
        IdentTopic::new(value.as_str())
    }
}

pub struct GossipsubMessage {
    pub topic: IdentTopic,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkingMessage {
    Multiaddr(Multiaddr),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MarketMessage {
    Job(Job),
    JobBid(JobBid),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DelegationMessage {
    Delegate(JobDelegation),
    Finished(JobWitness),
}

impl SwarmRunner {
    pub fn new(
        p2p_keypair: Keypair,
        p2p_multiaddr: Multiaddr,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut swarm = SwarmBuilder::with_existing_identity(p2p_keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().port_reuse(true),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|p2p_keypair| PeerBehaviour {
                gossipsub: Self::init_gossip(p2p_keypair).unwrap(),
            })?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(Topic::Networking.as_str()))?;
        swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(Topic::Market.as_str()))?;
        swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(Topic::Delegation.as_str()))?;
        // swarm.listen_on("/ip4/0.0.0.0/udp/5678/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/5679".parse()?)?;

        Ok(SwarmRunner { swarm, p2p_multiaddr })
    }

    fn init_gossip(
        p2p_local_keypair: &Keypair,
    ) -> Result<gossipsub::Behaviour, Box<dyn std::error::Error>> {
        let message_authenticity =
            gossipsub::MessageAuthenticity::Signed(p2p_local_keypair.clone());

        let config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .max_transmit_size(usize::MAX)
            .build()?;

        Ok(gossipsub::Behaviour::new(message_authenticity, config)?)
    }

    pub fn run(
        mut self,
        mut gossipsub_message: mpsc::Receiver<GossipsubMessage>,
    ) -> Pin<Box<dyn Stream<Item = PeerBehaviourEvent> + Send>> {
        let stream = stream! {
            loop {
                tokio::select! {
                    Some(message) = gossipsub_message.recv() => {
                        debug!{"Sending gossipsub_message: topic {}, data {:?}", message.topic, message.data};
                        if let Err(e) = self.swarm
                            .behaviour_mut()
                            .gossipsub
                            .publish(message.topic, message.data)
                        {
                            error!("Publish error: {e:?}");
                        }
                    },
                    event = self.swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(PeerBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed {
                            peer_id, topic
                        })) => {
                            if topic == Topic::Networking.into() {
                                self.swarm.behaviour_mut().gossipsub.publish(
                                    Topic::Networking,
                                    serde_json::to_vec(&NetworkingMessage::Multiaddr(self.p2p_multiaddr.to_owned())).unwrap()
                                ).unwrap();
                            }

                            yield PeerBehaviourEvent::Gossipsub(gossipsub::Event::Subscribed {
                                peer_id, topic
                            });
                        },
                        SwarmEvent::Behaviour(PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                            propagation_source,
                            message_id,
                            message,
                        })) => {
                            if message.topic == Topic::Networking.into() {
                                match serde_json::from_slice::<NetworkingMessage>(&message.data) {
                                    Ok(NetworkingMessage::Multiaddr(addr)) => {
                                        if let Err(error) = self.swarm.dial(addr) {
                                            error!{"Dial error: {:?}", error};
                                        }
                                    }
                                    Err(error) => {
                                        error!{"Deserialization error: {:?}", error};
                                    }
                                }
                            }

                            yield PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                                propagation_source,
                                message_id,
                                message,
                            });
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, connection_id, num_established, .. } => {
                            info!{"Connection established: peer_id {}, connection_id {}, num_established {}", peer_id, connection_id, num_established};
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                        SwarmEvent::ConnectionClosed { peer_id, connection_id, num_established, .. } => {
                            info!{"Connection closed: peer_id {}, connection_id {}, num_established {}", peer_id, connection_id, num_established};
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                        SwarmEvent::Behaviour(event) => {
                            yield event;
                        }
                        event => {
                            debug!("Unhandled event: {:?}", event);
                        }
                    },
                    _ = shutdown_signal() => {
                        break
                    }
                    else => break
                }
            }
        };
        Box::pin(stream)
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Serde error")]
    Serde(#[from] serde_json::Error),

    #[error("Dial error")]
    Dial(#[from] DialError),
}
