#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    /// Starknet mainnet.
    Mainnet,
    /// Sepolia testnet.
    Sepolia,
}

pub fn get_network_id(network: Network) -> &'static str {
    match network {
        Network::Mainnet => "mainnet",
        Network::Sepolia => "sepolia",
    }
}
