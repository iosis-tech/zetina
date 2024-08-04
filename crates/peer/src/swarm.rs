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
use tracing::{debug, error, info};
use zetina_common::graceful_shutdown::shutdown_signal;

#[derive(NetworkBehaviour)]
pub struct PeerBehaviour {
    gossipsub: gossipsub::Behaviour,
}

pub struct SwarmRunner {
    pub swarm: Swarm<PeerBehaviour>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Networking,
}

impl Topic {
    pub fn as_str(&self) -> &'static str {
        match self {
            Topic::Networking => "networking",
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

impl SwarmRunner {
    pub fn new(p2p_local_keypair: &Keypair) -> Result<Self, Box<dyn std::error::Error>> {
        let gossipsub = Self::init_gossip(p2p_local_keypair)?;
        let behaviour = PeerBehaviour { gossipsub };
        let local_keypair = p2p_local_keypair.clone();
        let mut swarm = SwarmBuilder::with_existing_identity(local_keypair)
            .with_tokio()
            .with_tcp(
                tcp::Config::default().port_reuse(true),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_quic()
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        swarm.behaviour_mut().gossipsub.subscribe(&IdentTopic::new(Topic::Networking.as_str()))?;
        // swarm.listen_on("/ip4/0.0.0.0/udp/5678/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/5679".parse()?)?;

        Ok(SwarmRunner { swarm })
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

    pub fn run(mut self) -> Pin<Box<dyn Stream<Item = PeerBehaviourEvent> + Send>> {
        let stream = stream! {
            loop {
                tokio::select! {
                    event = self.swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                            propagation_source,
                            message_id,
                            message,
                        })) => {
                            debug!("Gossipsub event: {:?}, {:?}, {:?}", propagation_source, message_id, message);
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
                            debug!("Behaviour event: {:?}", event);
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

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkingMessage {
    Multiaddr(Multiaddr),
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Serde error")]
    Serde(#[from] serde_json::Error),

    #[error("Dial error")]
    Dial(#[from] DialError),
}
