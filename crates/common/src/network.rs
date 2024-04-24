/*
    Network
    This defines the network that the node is connected to.
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
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
