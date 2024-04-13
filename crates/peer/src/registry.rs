use std::error::Error;

use starknet::{
    core::types::{BlockId, EmittedEvent, EventFilter, FieldElement},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, Url},
};

pub struct RegistryHandler {
    pub provider: JsonRpcClient<HttpTransport>,
    address: FieldElement,
}

impl RegistryHandler {
    pub fn new(url: &str, address: &str) -> Self {
        let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(url).unwrap()));
        let address = FieldElement::from_hex_be(address).unwrap();
        Self { provider, address }
    }

    async fn scrape_event(
        &self,
        event_keys: Vec<String>,
        from_block: u64,
    ) -> Result<Vec<EmittedEvent>, Box<dyn Error>> {
        let keys = event_keys
            .iter()
            .map(|key| FieldElement::from_hex_be(key))
            .collect::<Result<Vec<FieldElement>, _>>()?;

        let latest_block_number = self.provider.block_number().await?;

        let filter = EventFilter {
            from_block: Some(BlockId::Number(from_block)),
            to_block: Some(BlockId::Number(latest_block_number)),
            address: Some(self.address),
            keys: Some(vec![keys.clone()]),
        };

        let events = self.provider.get_events(filter, None, 1000).await?.events;
        Ok(events)
    }

    pub async fn run(&self) {
        // Create an interval of every 5 seconds
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;

            println!("Scraping events...");

            // Scrape the event
            let result = self
                .scrape_event(
                    vec!["0x17ef19eae2188756c1689ef60586c692a3aee6fecc18ee1b21f3028f75b9988"
                        .to_string()],
                    0,
                )
                .await;

            // Handle the result
            match result {
                Ok(events) => {
                    println!("{} Events Found", events.len());
                }
                Err(e) => {
                    eprintln!("Error scraping events: {:?}", e);
                }
            }
        }
    }
}
