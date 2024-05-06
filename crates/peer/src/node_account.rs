use std::error::Error;

use crypto_bigint::U256;
use sharp_p2p_common::{job_witness::JobWitness, network::Network};
use starknet::{
    accounts::{Account, Call, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
    core::types::{BlockId, BlockTag, FieldElement, FunctionCall},
    macros::selector,
    providers::Provider,
    signers::{LocalWallet, SigningKey, VerifyingKey},
};
use tracing::trace;

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
    ///
    ///
    signing_key: SigningKey,
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
        let signer = LocalWallet::from(signing_key.clone());
        let address = FieldElement::from_byte_slice_be(address.as_slice()).unwrap();
        let network = network.to_field_element();
        let account =
            SingleOwnerAccount::new(provider, signer, address, network, ExecutionEncoding::New);

        Self { p2p_keypair, account, signing_key }
    }

    pub fn get_keypair(&self) -> &libp2p::identity::Keypair {
        &self.p2p_keypair
    }

    pub fn get_account(&self) -> &SingleOwnerAccount<P, LocalWallet> {
        &self.account
    }

    pub fn get_provider(&self) -> &P {
        self.account.provider()
    }

    pub fn get_signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    pub fn get_verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    pub async fn deposit(
        &self,
        amount: FieldElement,
        registry_address: FieldElement,
    ) -> Result<(), Box<dyn Error>> {
        let result = self
            .account
            .execute(vec![Call {
                to: registry_address,
                selector: selector!("deposit"),
                calldata: vec![amount],
            }])
            .send()
            .await
            .unwrap();

        trace!("Deposit result: {:?}", result);
        Ok(())
    }

    pub async fn balance(&self, registry_address: FieldElement) -> Result<U256, Box<dyn Error>> {
        let account_address = self.account.address();
        let call_result = self
            .get_provider()
            .call(
                FunctionCall {
                    contract_address: registry_address,
                    entry_point_selector: selector!("balance"),
                    calldata: vec![account_address],
                },
                BlockId::Tag(BlockTag::Latest),
            )
            .await
            .expect("failed to call contract");

        let low: u128 = call_result[0].try_into().unwrap();
        let high: u128 = call_result[1].try_into().unwrap();
        let call_result = U256::from(high << 128 | low);
        trace!("Balance result: {:?}", call_result);

        Ok(call_result)
    }

    pub async fn withdraw(
        &self,
        amount: FieldElement,
        registry_address: FieldElement,
    ) -> Result<(), Box<dyn Error>> {
        let result = self
            .account
            .execute(vec![Call {
                to: registry_address,
                selector: selector!("withdraw"),
                calldata: vec![amount],
            }])
            .send()
            .await
            .unwrap();

        trace!("Withdraw result: {:?}", result);
        Ok(())
    }

    pub async fn verify_job_witness(
        &self,
        registry_address: FieldElement,
        job_withness: JobWitness,
    ) -> Result<(), Box<dyn Error>> {
        let result = self
            .account
            .execute(vec![Call {
                to: registry_address,
                selector: selector!("verify_job_witness"),
                calldata: job_withness.proof,
            }])
            .send()
            .await
            .unwrap();

        trace!("Verify job witness result: {:?}", result);
        Ok(())
    }
}
