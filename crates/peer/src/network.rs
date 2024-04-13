#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    /// Starknet mainnet.
    Mainnet,
    /// Sepolia testnet.
    Sepolia,
}

impl Network {
    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Sepolia => "sepolia",
        }
    }
}
