use async_stream::try_stream;
use futures_core::stream::Stream;
use starknet::{
    core::types::{BlockId, EmittedEvent, EventFilter, FieldElement},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, Url},
};
use std::{error::Error, pin::Pin};
use tracing::trace;

const EVENT_SCRAPE_INTERVAL: u64 = 2;

pub struct RegistryHandler {
    pub provider: JsonRpcClient<HttpTransport>,
    contract_address: FieldElement,
    last_block_number: u64,
}

impl RegistryHandler {
    pub fn new(url: &str, contract_address: &str) -> Self {
        let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(url).unwrap()));
        let contract_address = FieldElement::from_hex_be(contract_address).unwrap();
        Self { provider, contract_address, last_block_number: 0 }
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
            address: Some(self.contract_address),
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
