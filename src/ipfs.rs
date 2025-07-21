use anyhow::Result;
use bytes::Bytes;
use reqwest;
use serde_json::Value;

pub struct IpfsManager {
    client: reqwest::Client,
    api_url: String,
}

impl IpfsManager {
    pub fn new(api_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_url: api_url.to_string(),
        }
    }

    pub async fn add(&self, data: Bytes) -> Result<String> {
        let form = reqwest::multipart::Form::new().part("file", reqwest::multipart::Part::bytes(data.to_vec()));
        let res = self
            .client
            .post(&format!("{}/api/v0/add", self.api_url))
            .multipart(form)
            .send()
            .await?;
        let json: Value = res.json().await?;
        Ok(json["Hash"].as_str().unwrap().to_string())
    }

    pub async fn get(&self, hash: &str) -> Result<Bytes> {
        let res = self
            .client
            .post(&format!("{}/api/v0/cat?arg={}", self.api_url, hash))
            .send()
            .await?;
        Ok(res.bytes().await?)
    }
}
