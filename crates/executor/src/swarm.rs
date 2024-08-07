use futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::identity::Keypair;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{noise, tcp, yamux, Multiaddr, SwarmBuilder};
use std::error::Error;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};
use zetina_common::graceful_shutdown::shutdown_signal;

#[derive(NetworkBehaviour)]
pub struct PeerBehaviour {
    gossipsub: gossipsub::Behaviour,
}

pub struct SwarmRunner {
    handle: Option<JoinHandle<()>>,
}

impl SwarmRunner {
    pub fn new(
        dial_addresses: Vec<String>,
        p2p_local_keypair: &Keypair,
        subscribe_topics: Vec<IdentTopic>,
        mut transmit_topics: Vec<(IdentTopic, mpsc::Receiver<Vec<u8>>)>,
        swarm_events_tx: mpsc::Sender<gossipsub::Event>,
    ) -> Result<Self, Box<dyn Error>> {
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

        for topic in subscribe_topics {
            swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        }

        swarm.listen_on("/ip4/0.0.0.0/udp/5678/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/5679".parse()?)?;

        // Reach out to other nodes if specified
        for to_dial in dial_addresses {
            let addr: Multiaddr = Multiaddr::from_str(&to_dial)?;
            swarm.dial(addr)?;
            info!("Dialed {to_dial:?}")
        }

        Ok(SwarmRunner {
            handle: Some(tokio::spawn(async move {
                // TODO make it nicer solution, extensible not manual!
                let mut topic_one = transmit_topics.pop().unwrap();
                let mut topic_two = transmit_topics.pop().unwrap();
                loop {
                    tokio::select! {
                        Some(data) = topic_one.1.recv() => {
                            debug!("Publishing to topic: {:?}", topic_one.0);
                            if let Err(e) = swarm
                                .behaviour_mut().gossipsub
                                .publish(topic_one.0.clone(), data) {
                                error!("Publish error: {e:?}");
                            }
                        },
                        Some(data) = topic_two.1.recv() => {
                            debug!("Publishing to topic: {:?}", topic_two.0);
                            if let Err(e) = swarm
                                .behaviour_mut().gossipsub
                                .publish(topic_two.0.clone(), data) {
                                error!("Publish error: {e:?}");
                            }
                        },
                        event = swarm.select_next_some() => match event {
                            SwarmEvent::Behaviour(PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                                propagation_source,
                                message_id,
                                message,
                            })) => {
                                swarm_events_tx.send(gossipsub::Event::Message { propagation_source, message_id, message }).await.unwrap();
                            },
                            SwarmEvent::NewListenAddr { address, .. } => {
                                debug!("Local node is listening on {address}");
                            }
                            SwarmEvent::ConnectionEstablished { peer_id, connection_id, num_established, .. } => {
                                info!{"ConnectionEstablished: peer_id {}, connection_id {}, num_established {}", peer_id, connection_id, num_established};
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            }
                            SwarmEvent::ConnectionClosed { peer_id, connection_id, num_established, .. } => {
                                info!{"ConnectionClosed: peer_id {}, connection_id {}, num_established {}", peer_id, connection_id, num_established};
                                swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                            }
                            _ => {}
                        },
                        _ = shutdown_signal() => {
                            break
                        }
                        else => break
                    }
                }
            })),
        })
    }

    fn init_gossip(p2p_local_keypair: &Keypair) -> Result<gossipsub::Behaviour, Box<dyn Error>> {
        let message_authenticity =
            gossipsub::MessageAuthenticity::Signed(p2p_local_keypair.clone());

        let config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .max_transmit_size(usize::MAX)
            .build()?;

        Ok(gossipsub::Behaviour::new(message_authenticity, config)?)
    }
}

impl Drop for SwarmRunner {
    fn drop(&mut self) {
        let handle = self.handle.take();
        tokio::spawn(async move {
            if let Some(handle) = handle {
                handle.await.unwrap();
            }
        });
    }
}
