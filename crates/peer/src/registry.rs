use async_stream::try_stream;
use futures::stream::Stream;
use starknet::{
    core::types::{BlockId, EmittedEvent, EventFilter, FieldElement},
    providers::Provider,
};
use std::{error::Error, pin::Pin};
use tracing::trace;

const EVENT_SCRAPE_INTERVAL: u64 = 2;
const REGISTRY_CONTRACT: &str =
    "0x030938966f24f5084d9570ac52aeff76fe30559f4f3fe086a2b0cb4017ce4384";

/*
    Registry Handler
    This object is responsible for handle continuous scraping of events from the Registry contract.
    It scrapes the events from the Registry contract and provides a stream of events.
*/

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
}
