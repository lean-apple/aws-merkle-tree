#[cfg(test)]
mod tests {

    use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
    use aws_merkle_tree::{handler::handler, merkle::*};
    use aws_sdk_dynamodb::Client;
    use lambda_http::http::{HeaderMap, Method, Request};

    const AWS_REGION: &str = "eu-west-3";
    const MERKLE_TREE_TABLE: &str = "DevMerkleTree";

    #[tokio::test]
    async fn global_merkle_flow_ok() {
        
        // Setup region config for client
        let region_provider = RegionProviderChain::default_provider().or_else(AWS_REGION);
        let config = aws_config::defaults(BehaviorVersion::v2023_11_09())
            .region(region_provider)
            .load()
            .await;
        let dynamodb_client = Client::new(&config);

        // 8 very basic leaves "children" data to build a 15-leave tree
        let leaves = [
            "leaf0", "leaf1", "leaf2", "leaf3", "leaf4", "leaf5", "leaf6", "leaf7",
        ];

        // Check DevMekrleTree table is in the DynamoDB list
        assert!(list_tables(&dynamodb_client).await.is_ok());

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
        assert!(is_valid_merkle_tree(&nodes));

        // Check we can get the node infos for the index 7 -
        // TODO: Change to assert_eq on expected values
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

    #[tokio::test]
    async fn test_node_get_api() {
        let mut query_string_parameters = HeaderMap::new();
        query_string_parameters.insert("index", "7".parse().unwrap());

        let request = Request::builder()
            .method(Method::GET)
            .uri("https://example.com?index=9")
            .header("Content-Type", "application/json")
            .body(lambda_http::Body::Empty)
            .expect("Failed to build request");

        let response = handler(request).await.expect("Handler failed");

        // Inspect the response
        assert!(
            response.status().is_success(),
            "Response was not successful"
        );
    }
}
