use std::error::Error;

use libp2p::futures::StreamExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::multiaddr::Protocol;
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::{mdns, noise, tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder};
use tokio::io::{self, AsyncBufReadExt};

use crate::network::Network;
use crate::node::{NodeConfig, NodeType};

#[derive(NetworkBehaviour)]
struct PeerBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

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

pub(crate) fn gossipsub_ident_topic(network: Network, topic: Topic) -> IdentTopic {
    let network = network.as_str();
    let topic = topic.as_str();
    let s = format!("/{network}/{topic}");
    IdentTopic::new(s)
}

pub struct SwarmRunner {
    swarm: Swarm<PeerBehaviour>,
    network: Network,
}

impl SwarmRunner {
    pub fn new(node_config: &NodeConfig) -> Result<Self, Box<dyn Error>> {
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            node_config.p2p_local_keypair.public().to_peer_id(),
        )?;
        let gossipsub = init_gossip(node_config)?;
        let behaviour = PeerBehaviour { gossipsub, mdns };
        let local_keypair = node_config.p2p_local_keypair.clone();
        let mut swarm = SwarmBuilder::with_existing_identity(local_keypair)
            .with_tokio()
            .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
            .with_quic()
            .with_behaviour(|_| behaviour)
            .expect("Moving behaviour doesn't fail")
            .build();

        swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        Ok(SwarmRunner { swarm, network: node_config.network })
    }

    pub async fn run(&mut self, node_type: NodeType) {
        // Read full lines from stdin
        let mut stdin = io::BufReader::new(io::stdin()).lines();

        let publish_topic = match node_type {
            NodeType::Delegator => gossipsub_ident_topic(self.network, Topic::NewJob),
            NodeType::Executor => gossipsub_ident_topic(self.network, Topic::PickedJob),
        };

        loop {
            tokio::select! {
                Ok(Some(line)) = stdin.next_line() => {
                    println!("Publishing to topic: {:?}", publish_topic);
                    if let Err(e) = self.swarm
                        .behaviour_mut().gossipsub
                        .publish(publish_topic.clone(), line.as_bytes()) {
                        println!("Publish error: {e:?}");
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discovered a new peer: {peer_id}");
                            self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(PeerBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _multiaddr) in list {
                            println!("mDNS discover peer has expired: {peer_id}");
                            self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        }
                    },
                    SwarmEvent::Behaviour(PeerBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => println!(
                            "Got message: '{}' with id: {id} from peer: {peer_id}",
                            String::from_utf8_lossy(&message.data),
                        ),
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Local node is listening on {address}");
                    }
                    _ => {}
                }
            }
        }
    }
}

fn init_gossip(node_config: &NodeConfig) -> Result<gossipsub::Behaviour, Box<dyn Error>> {
    let message_authenticity =
        gossipsub::MessageAuthenticity::Signed(node_config.p2p_local_keypair.clone());
    let config = gossipsub::ConfigBuilder::default()
        .validation_mode(gossipsub::ValidationMode::Strict)
        .validate_messages()
        .build()
        .unwrap();
    let mut gossipsub: gossipsub::Behaviour =
        gossipsub::Behaviour::new(message_authenticity, config).unwrap();

    // `new-job` is the topic about new job to be proven
    let new_job_topic = gossipsub_ident_topic(node_config.network, Topic::NewJob);
    // `picked-job` is the topic about picked job to processing prover
    let picked_job_topic = gossipsub_ident_topic(node_config.network, Topic::PickedJob);

    match node_config.node_type {
        NodeType::Delegator => {
            println!("Delegator: Subscribed no topic");
        }
        NodeType::Executor => {
            gossipsub.subscribe(&picked_job_topic)?;
            gossipsub.subscribe(&new_job_topic)?;
            println!("Executor: Subscribed to topic: {:?}, {:?}", new_job_topic, picked_job_topic);
        }
    }

    Ok(gossipsub)
}

pub(crate) trait MultiaddrExt {
    fn peer_id(&self) -> Option<PeerId>;
}

impl MultiaddrExt for Multiaddr {
    fn peer_id(&self) -> Option<PeerId> {
        self.iter().find_map(|proto| match proto {
            Protocol::P2p(peer_id) => Some(peer_id),
            _ => None,
        })
    }
}
