use sharp_p2p_peer::network::Network;
use sharp_p2p_peer::node::{Node, NodeConfig, NodeType};
use sharp_p2p_peer::store::Store;
use std::error::Error;

use std::time::Duration;
use tokio::time::sleep;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).try_init();

    // 1. Generate keypair for the node
    let p2p_local_keypair = libp2p::identity::Keypair::generate_ed25519();

    // 2. Initiate a new node to sync with other peers
    let store = Store::new();
    let node_config =
        NodeConfig::new(NodeType::Executor, Network::Sepolia, p2p_local_keypair, Vec::new(), store);
    let node = Node::new(node_config).await.unwrap();
    println!("node: {:?}", node);

    loop {
        sleep(Duration::from_secs(1)).await;
    }
}
