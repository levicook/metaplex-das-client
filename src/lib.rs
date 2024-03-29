use std::sync::Arc;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

pub type RateLimiter = Arc<
    governor::RateLimiter<
        governor::state::NotKeyed,
        governor::state::InMemoryState,
        governor::clock::DefaultClock,
        governor::middleware::NoOpMiddleware,
    >,
>;

pub struct DasClient {
    url: String,
    client: reqwest::Client,
    limiter: RateLimiter,
}

impl DasClient {
    pub fn new<U: ToString>(url: U, client: reqwest::Client, limiter: RateLimiter) -> Self {
        Self {
            url: url.to_string(),
            client,
            limiter,
        }
    }

    pub async fn get_asset(&self, asset_id: &str) -> Result<Asset> {
        self.send("getAsset", serde_json::json!({"id": asset_id}))
            .await
    }

    pub async fn get_assets_by_owner(
        &self,
        owner_address: &str,
        page: u16,
        limit: u16,
        // displayOptions?
    ) -> Result<Asset> {
        self.send(
            "getAssetsByOwner",
            serde_json::json!({
                "ownerAddress": owner_address,
                "page": page,
                "limit": limit,
                "displayOptions": { // TODO(Levi) consider parameterizing this
                    "showFungible": true,
                }
            }),
        )
        .await
    }

    async fn send<T>(&self, method: &str, params: serde_json::Value) -> Result<T>
    where
        T: serde::de::DeserializeOwned, // Ensure T can be deserialized
    {
        self.limiter.until_ready().await;

        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "my-id", // TODO(Levi) care?
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.text().await?;
            println!("{}", body);

            let body = serde_json::from_str::<Response<T>>(&body)?;
            return Ok(body.result);
        }

        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to read response body".to_string());

        panic!(
            // TODO(Levi) turn this into an error
            "Request failed with status: {}\nResponse body: {}",
            status, body
        );
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Response<T> {
    pub jsonrpc: String,
    pub id: String,
    pub result: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Asset {
    pub interface: Option<String>,
    pub id: Option<String>,
    pub content: Option<Content>,
    pub authorities: Option<Vec<Authority>>,
    pub compression: Option<Compression>,
    pub grouping: Option<Vec<Grouping>>,
    pub royalty: Option<Royalty>,
    pub creators: Option<Vec<Creator>>,
    pub ownership: Option<Ownership>,
    pub uses: Option<Uses>,
    pub supply: Option<Supply>,
    pub mutable: Option<bool>,
    pub burnt: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Content {
    pub json_uri: String,
    pub files: Vec<File>,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub uri: String,
    pub mime: String,
    pub contexts: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub token_standard: String,
    pub attributes: Option<Vec<Attribute>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub value: String,
    pub trait_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Authority {
    pub address: String,
    pub scopes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Compression {
    pub eligible: bool,
    pub compressed: bool,
    pub data_hash: String,
    pub creator_hash: String,
    pub asset_hash: String,
    pub tree: String,
    pub seq: u64,
    pub leaf_id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Grouping {
    pub group_key: String,
    pub group_value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Royalty {
    pub royalty_model: String,
    pub target: Option<String>,
    pub percent: f64,
    pub basis_points: u32,
    pub primary_sale_happened: bool,
    pub locked: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Creator {
    pub address: String,
    pub share: f64, // Assuming this is a floating-point number representing the share percentage
    pub verified: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ownership {
    pub frozen: bool,
    pub delegated: bool,
    pub delegate: Option<String>,
    pub ownership_model: String,
    pub owner: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Uses {
    pub use_method: String,
    pub remaining: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Supply {
    pub print_max_supply: u64,
    pub print_current_supply: u64,
    pub edition_nonce: u64,
}
