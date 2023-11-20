use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{Client, Error};

mod merkle;
use merkle::*;
const AWS_REGION: &str = "eu-west-3";
const MERKLE_TREE_TABLE: &str = "DevMerkleTree";

#[tokio::main]
async fn main() {
    let region_provider = RegionProviderChain::default_provider().or_else(AWS_REGION);
    let sdk_config: aws_types::SdkConfig = aws_config::defaults(BehaviorVersion::v2023_11_09())
        .region(region_provider)
        .load()
        .await;

    let dynamodb = Client::new(&sdk_config as &aws_types::sdk_config::SdkConfig);
    let leaves = ["leaf1", "leaf2", "leaf3", "leaf4"];

    list_tables(&dynamodb)
        .await
        .expect("tables could not be listed");

    match create_and_store_merkle_tree(&dynamodb, MERKLE_TREE_TABLE, &leaves).await {
        Ok(_) => println!("Merkle tree created and stored successfully."),
        Err(e) => eprintln!("Error creating and storing Merkle tree: {:?}", e),
    }

    // Fetch Merkle tree from DynamoDB
    let nodes = fetch_merkle_tree_from_db(&dynamodb, MERKLE_TREE_TABLE)
        .await
        .expect("Failed to fetch mekrle tree nodes from DynamoDB");

    // Validate Merkle tree
    if is_valid_merkle_tree(&nodes) {
        println!("Merkle tree is valid.");
    } else {
        println!("Merkle tree is invalid.");
    }
}

async fn list_tables(client: &Client) -> Result<(), Error> {
    let resp = client.list_tables().send().await?;
    println!("Tables: {:?}", resp.table_names());
    Ok(())
}
