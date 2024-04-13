use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use std::error::Error;
use std::sync::Arc;

use crate::network::Network;
use crate::registry::RegistryHandler;
use crate::store::Store;
use crate::swarm::SwarmRunner;

pub enum NodeType {
    Delegator,
    Executor,
}

pub struct NodeConfig {
    pub node_type: NodeType,
    /// An id of the network to connect to.
    pub network: Network,
    /// The keypair to be used as [`Node`]s identity.
    pub p2p_local_keypair: Keypair,
    /// List of the addresses where [`Node`] will listen for incoming connections.
    pub p2p_listen_on: Vec<Multiaddr>,
    /// The store for job record.
    pub store: Store,
}

impl NodeConfig {
    pub fn new(
        node_type: NodeType,
        network: Network,
        p2p_local_keypair: Keypair,
        p2p_listen_on: Vec<Multiaddr>,
        store: Store,
    ) -> Self {
        Self { node_type, network, p2p_local_keypair, p2p_listen_on, store }
    }
}

#[derive(Debug)]
pub struct Node {
    pub store: Arc<Store>,
}

impl Node {
    pub async fn new(node_config: NodeConfig) -> Result<Self, Box<dyn Error>> {
        let mut swarm_runner = SwarmRunner::new(&node_config)?;
        let registry_handler = RegistryHandler::new(
            "https://starknet-sepolia.public.blastapi.io",
            "0xcdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b",
        );
        // Node should run swarm runner and registry handler concurrently.
        tokio::spawn(async move {
            swarm_runner.run(node_config.node_type).await;
        });
        tokio::spawn(async move {
            registry_handler.run().await;
        });

        let store = Arc::new(node_config.store);

        Ok(Self { store })
    }
}
