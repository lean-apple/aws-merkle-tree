use crate::merkle::get_node_info_from_db;
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;
use lambda_http::{http::Response, Error, Request};
use serde_json::json;
use url::{ParseError, Url};

const AWS_REGION: &str = "eu-west-3";
const MERKLE_TREE_TABLE: &str = "DevMerkleTree";

pub async fn handler(event: Request) -> Result<Response<lambda_http::Body>, Error> {
    // Extract parameters from the request
    let query_params = event.into_parts();
    let query_index = extract_index_from_url(&query_params.0.uri.to_string())
        .expect("Could not find index in URI");

    // Setup region config for client
    let region_provider = RegionProviderChain::default_provider().or_else(AWS_REGION);
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region_provider)
        .load()
        .await;
    let dynamodb_client = Client::new(&config.into());

    match get_node_info_from_db(&dynamodb_client, &MERKLE_TREE_TABLE, query_index).await {
        Ok((depth, offset, hash)) => {

            print!("hheloo3 depth {:?}", depth);
            print!("hheloo3 offset {:?}", offset);
            print!("hheloo3 hash {:?}", hash);
            // Create a successful response
            Ok(Response::builder()
                .status(200)
                .body(
                    json!({
                        "depth": depth,
                        "offset": offset,
                        "hash": hash
                    })
                    .to_string()
                    .into(),
                )
                .expect("Failed to render response"))
        }
        Err(e) => {
            // Handle error case
            Ok(Response::builder()
                .status(500)
                .body(format!("Error: {:?}", e).into())
                .expect("Failed to render error response"))
        }
    }
}

// Helper to get index from URI
fn extract_index_from_url(url: &str) -> Result<u32, String> {
    let parsed_url = Url::parse(url).map_err(|e: ParseError| e.to_string())?;

    parsed_url
        .query_pairs()
        .find_map(|(key, value)| {
            if key == "index" {
                value.parse::<u32>().ok()
            } else {
                None
            }
        })
        .ok_or("Index parameter not found or not a valid u32".to_string())
}
