use ad4m_client::Client;
use anyhow::Result;

#[derive(Debug)]
pub struct Ad4mManager {
    client: Client,
}

impl Ad4mManager {
    pub async fn new() -> Result<Self> {
        let client = Client::new("http://localhost:4000").await?;
        Ok(Self { client })
    }
}
