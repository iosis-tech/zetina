use starknet::{
    core::types::FieldElement,
    signers::{SigningKey, VerifyingKey},
};

pub struct NodeAccount
{
    /// Key pair for the p2p network.
    /// This represents the identity of the node in the network.
    p2p_keypair: libp2p::identity::Keypair,
    signing_key: SigningKey,
}

impl NodeAccount
{
    pub fn new(private_key: Vec<u8>) -> Self {
        let _secret_key =
            libp2p::identity::ecdsa::SecretKey::try_from_bytes(private_key.as_slice())
                .expect("Failed to create secret key from private key.");
        let p2p_keypair =
            libp2p::identity::Keypair::from(libp2p::identity::ecdsa::Keypair::generate());
        let signing_key = SigningKey::from_secret_scalar(
            FieldElement::from_byte_slice_be(private_key.as_slice()).unwrap(),
        );

        Self { p2p_keypair, signing_key }
    }

    pub fn get_keypair(&self) -> &libp2p::identity::Keypair {
        &self.p2p_keypair
    }

    pub fn get_signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    pub fn get_verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}
