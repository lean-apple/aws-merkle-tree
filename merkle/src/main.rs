use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::{Client, Error};
use aws_config::BehaviorVersion;

mod merkle;
use merkle::*;

const AWS_REGION: &str = "eu-west-3";
const MERKLE_TREE_TABLE: &str = "DevMerkleTree";

#[tokio::main]
async fn main() {
    // Setup region config for client
    let region_provider = RegionProviderChain::default_provider().or_else(AWS_REGION);
    let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
    .region(region_provider)
    .load().await;
    let dynamodb_client = Client::new(&config.into());

    // 8 very basic leaves "children" data to build a 15-leave tree
    let leaves = ["leaf0", "leaf1", "leaf2", "leaf3", "leaf4", "leaf5", "leaf6", "leaf7"];

    // Check DevMekrleTree table is in the DynamoDB list
    list_tables(&dynamodb_client)
        .await
        .expect("DynamoDB tables could not be listed");

    // Create store the Merkle Tree from raw leaves data
    match create_and_store_merkle_tree(&dynamodb_client, MERKLE_TREE_TABLE, &leaves).await {
        Ok(_) => println!("Merkle tree created and stored successfully."),
        Err(e) => eprintln!("Error creating and storing Merkle tree: {:?}", e),
    }

    // Fetch Merkle tree from DynamoDB
    let nodes = fetch_merkle_tree_from_db(&dynamodb_client, MERKLE_TREE_TABLE)
        .await
        .expect("Failed to fetch Merkle Tree nodes from AWS DynamoDB");

    // Check validity of Merkle tree
    if is_valid_merkle_tree(&nodes) {
        println!("Fetched Merkle tree is valid.");
    } else {
        eprintln!("Fetched Merkle tree is invalid.");
    }

    // Check we can get the node infos for the index 7 - On teh
    match get_node_info_from_db(&dynamodb_client, MERKLE_TREE_TABLE, 7).await {
        Ok((depth, offset, hash)) => {
            println!(
                "Node info - Depth: {}, Offset: {}, Hash: {}",
                depth, offset, hash
            );
        }
        Err(e) => {
            eprintln!("Error fetching node info: {:?}", e);
        }
    }
}

// List and print available DynamoDB tables
async fn list_tables(client: &Client) -> Result<(), Error> {
    let resp = client.list_tables().send().await?;
    println!("Tables: {:?}", resp.table_names());
    Ok(())
}