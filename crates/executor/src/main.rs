#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    zetina_executor::run().await
}
