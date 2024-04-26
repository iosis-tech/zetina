use async_stream::stream;
use futures_core::stream::Stream;
use libp2p::futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::identity::Keypair;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{mdns, noise, tcp, yamux, Swarm, SwarmBuilder};
use std::error::Error;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

#[derive(NetworkBehaviour)]
struct PeerBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

pub struct SwarmRunner {
    swarm: Swarm<PeerBehaviour>,
    cancellation_token: CancellationToken,
}

impl SwarmRunner {
    pub fn new(
        p2p_local_keypair: &Keypair,
        subscribe_topics: &[IdentTopic],
    ) -> Result<Self, Box<dyn Error>> {
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            p2p_local_keypair.public().to_peer_id(),
        )?;
        let gossipsub = Self::init_gossip(p2p_local_keypair)?;
        let behaviour = PeerBehaviour { gossipsub, mdns };
        let local_keypair = p2p_local_keypair.clone();
        let mut swarm = SwarmBuilder::with_existing_identity(local_keypair)
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_quic()
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        for topic in subscribe_topics {
            swarm.behaviour_mut().gossipsub.subscribe(topic)?;
        }

        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(SwarmRunner { swarm, cancellation_token: CancellationToken::new() })
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

    pub fn run(
        &mut self,
        send_topic: IdentTopic,
        mut send_topic_rx: mpsc::Receiver<Vec<u8>>,
    ) -> Pin<Box<impl Stream<Item = gossipsub::Event> + '_>> {
        let stream = stream! {
            loop {
                tokio::select! {
                    Some(data) = send_topic_rx.recv() => {
                        debug!("Publishing to topic: {:?}", send_topic);
                        if let Err(e) = self.swarm
                            .behaviour_mut().gossipsub
                            .publish(send_topic.clone(), data) {
                            error!("Publish error: {e:?}");
                        }
                    },
                    event = self.swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                            for (peer_id, _multiaddr) in list {
                                debug!("mDNS discovered a new peer: {peer_id}");
                                self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                            }
                        },
                        SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                            for (peer_id, _multiaddr) in list {
                                debug!("mDNS discover peer has expired: {peer_id}");
                                self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                            }
                        },
                        SwarmEvent::Behaviour(PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                            propagation_source,
                            message_id,
                            message,
                        })) => {
                            yield gossipsub::Event::Message { propagation_source, message_id, message };
                        },
                        SwarmEvent::NewListenAddr { address, .. } => {
                            debug!("Local node is listening on {address}");
                        }
                        _ => {}
                    },
                    _ = self.cancellation_token.cancelled() => {
                        break
                    }
                }
            }
        };
        Box::pin(stream)
    }
}

impl Drop for SwarmRunner {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
