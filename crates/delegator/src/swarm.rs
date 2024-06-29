pub mod event_loop;

use event_loop::swarm_loop;
use futures::executor::block_on;
use futures::FutureExt;
use libp2p::gossipsub::{self, IdentTopic};
use libp2p::identity::Keypair;
use libp2p::swarm::NetworkBehaviour;
use libp2p::{mdns, noise, tcp, yamux, SwarmBuilder};
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(NetworkBehaviour)]
pub struct PeerBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

pub struct SwarmRunner {
    cancellation_token: CancellationToken,
    handle: Option<JoinHandle<()>>,
}

impl SwarmRunner {
    pub fn new(
        p2p_local_keypair: &Keypair,
        subscribe_topics: Vec<IdentTopic>,
        transmit_topics: Vec<(IdentTopic, mpsc::Receiver<Vec<u8>>)>,
        swarm_events_tx: mpsc::Sender<gossipsub::Event>,
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
            swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        }

        swarm.listen_on("/ip4/0.0.0.0/udp/5678/quic-v1".parse()?)?;
        swarm.listen_on("/ip4/0.0.0.0/tcp/5679".parse()?)?;

        let cancellation_token = CancellationToken::new();

        Ok(SwarmRunner {
            cancellation_token: cancellation_token.to_owned(),
            handle: Some(tokio::spawn(async move {
                swarm_loop(swarm, transmit_topics, swarm_events_tx, cancellation_token)
                    .boxed()
                    .await
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

    pub async fn run() {}
}

impl Drop for SwarmRunner {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
        block_on(async move {
            self.handle.take().unwrap().await.unwrap();
        })
    }
}
