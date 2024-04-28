/*
    Network
    This defines the network that the node is connected to.
*/

use starknet::core::chain_id;
use starknet_crypto::FieldElement;

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

    pub fn to_field_element(&self) -> FieldElement {
        match self {
            Network::Mainnet => chain_id::MAINNET,
            Network::Sepolia => chain_id::SEPOLIA,
        }
    }
}
