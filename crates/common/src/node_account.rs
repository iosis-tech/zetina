use starknet::{
    accounts::{ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
    core::types::FieldElement,
    providers::Provider,
    signers::{LocalWallet, SigningKey},
};

use crate::network::Network;

pub struct NodeAccount<P>
where
    P: Provider + Sync + Send + 'static,
{
    /// Key pair for the p2p network.
    /// This represents the identity of the node in the network.
    p2p_keypair: libp2p::identity::Keypair,
    /// The account for the StarkNet network.
    /// This account is used to interact with the Registry contract.
    account: SingleOwnerAccount<P, LocalWallet>,
}

impl<P> NodeAccount<P>
where
    P: Provider + Sync + Send + 'static,
{
    pub fn new(private_key: Vec<u8>, address: Vec<u8>, network: Network, provider: P) -> Self {
        let secret_key = libp2p::identity::ecdsa::SecretKey::try_from_bytes(private_key.as_slice())
            .expect("Failed to create secret key from private key.");
        let p2p_keypair =
            libp2p::identity::Keypair::from(libp2p::identity::ecdsa::Keypair::from(secret_key));
        let signing_key = SigningKey::from_secret_scalar(
            FieldElement::from_byte_slice_be(private_key.as_slice()).unwrap(),
        );
        let signer = LocalWallet::from(signing_key);
        let address = FieldElement::from_byte_slice_be(address.as_slice()).unwrap();
        let network = network.to_field_element();
        let account =
            SingleOwnerAccount::new(provider, signer, address, network, ExecutionEncoding::New);

        Self { p2p_keypair, account }
    }

    pub fn get_keypair(&self) -> libp2p::identity::Keypair {
        self.p2p_keypair.clone()
    }

    pub fn get_account(&self) -> &SingleOwnerAccount<P, LocalWallet> {
        &self.account
    }

    pub fn get_provider(&self) -> &P {
        self.account.provider()
    }
}
