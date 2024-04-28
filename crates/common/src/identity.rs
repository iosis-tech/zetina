use starknet::{core::types::FieldElement, signers::SigningKey};

pub struct IdentityHandler {
    /// Key pair for the p2p network.
    /// This represents the identity of the node in the network.
    pub p2p_keypair: libp2p::identity::Keypair,
    /// The signing key for the StarkNet network.
    /// This is used to sign messages and transactions.
    pub signing_key: SigningKey,
}

impl IdentityHandler {
    pub fn new(private_key: Vec<u8>) -> Self {
        let secret_key = libp2p::identity::ecdsa::SecretKey::try_from_bytes(private_key.as_slice())
            .expect("Failed to create secret key from private key.");
        let p2p_keypair =
            libp2p::identity::Keypair::from(libp2p::identity::ecdsa::Keypair::from(secret_key));
        let signing_key = SigningKey::from_secret_scalar(
            FieldElement::from_byte_slice_be(private_key.as_slice()).unwrap(),
        );
        Self { p2p_keypair, signing_key }
    }

    pub fn get_keypair(&self) -> libp2p::identity::Keypair {
        self.p2p_keypair.clone()
    }

    pub fn get_signing_key(&self) -> SigningKey {
        self.signing_key.clone()
    }
}
