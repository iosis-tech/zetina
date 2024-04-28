use async_stream::try_stream;
use futures::stream::Stream;
use starknet::{
    accounts::{Account, Call, SingleOwnerAccount},
    core::{
        types::{BlockId, EmittedEvent, EventFilter, FieldElement},
        utils::get_selector_from_name,
    },
    providers::Provider,
    signers::Signer,
};
use std::{error::Error, pin::Pin};
use tracing::trace;

const EVENT_SCRAPE_INTERVAL: u64 = 2;
const REGISTRY_CONTRACT: &str = "0xcdd51fbc4e008f4ef807eaf26f5043521ef5931bbb1e04032a25bd845d286b";

pub struct RegistryHandler<P> {
    provider: P,
    registry_address: FieldElement,
    last_block_number: u64,
}

impl<P> RegistryHandler<P>
where
    P: Provider + Sync + Send + 'static,
{
    pub fn new(provider: P) -> Self {
        let registry_address = FieldElement::from_hex_be(REGISTRY_CONTRACT).unwrap();
        Self { provider, registry_address, last_block_number: 0 }
    }

    pub fn get_provider(&self) -> &P {
        &self.provider
    }

    pub fn get_registry_address(&self) -> FieldElement {
        self.registry_address
    }

    async fn scrape_event(
        &mut self,
        event_keys: Vec<String>,
    ) -> Result<Vec<EmittedEvent>, Box<dyn Error>> {
        let keys = event_keys
            .iter()
            .map(|key| FieldElement::from_hex_be(key))
            .collect::<Result<Vec<FieldElement>, _>>()?;

        let latest_block_number = self.provider.block_number().await?;

        let filter = EventFilter {
            from_block: Some(BlockId::Number(self.last_block_number)),
            to_block: Some(BlockId::Number(latest_block_number)),
            address: Some(self.registry_address),
            keys: Some(vec![keys.clone()]),
        };

        let events = self.provider.get_events(filter, None, 1000).await?.events;
        self.last_block_number = latest_block_number;
        Ok(events)
    }

    pub fn subscribe_events(
        &mut self,
        event_keys: Vec<String>,
    ) -> Pin<Box<impl Stream<Item = Result<Vec<EmittedEvent>, Box<dyn Error + '_>>>>> {
        let stream = try_stream! {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(EVENT_SCRAPE_INTERVAL));
            loop {
                interval.tick().await;
                trace!("Scraping events...");
                let events = self.scrape_event(event_keys.clone()).await?;
                if !events.is_empty() {
                    yield events
                }
            }
        };
        Box::pin(stream)
    }

    pub async fn stake<S>(
        &self,
        amount: FieldElement,
        account: SingleOwnerAccount<P, S>,
    ) -> Result<(), Box<dyn Error>>
    where
        S: Signer + Sync + Send + 'static,
    {
        let result = account
            .execute(vec![Call {
                to: self.registry_address,
                selector: get_selector_from_name("stake").unwrap(),
                calldata: vec![amount],
            }])
            .send()
            .await
            .unwrap();

        trace!("Stake result: {:?}", result);
        Ok(())
    }
}
