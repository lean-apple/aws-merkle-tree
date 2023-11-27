#[cfg(test)]
mod tests {

    use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
    use aws_merkle_tree::{db::*, handler::handler, merkle::*};
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

        let expected_hash_sorted = [
            "c0003b5e8cc95dcbd5abb767520da54a3d6eb78f7e2254ca380446150c250434",
            "073cd5d53df7713fd5466e8f45906131d8fb2b35503b8c4847278743cbebed79",
            "58ee3fe35386971c6cff39842f054184f14697f8e7d6b5bb1ca35fb86b8b61bc",
            "f9b5e44bf841fa0154502b136be13274027480e4476595cc3c008c035c335501",
            "79edb6e4b2e862109d8f36996025d2c5f8c6fec869532c854ec7ba1d61efdbe6",
            "dcaf6a9eca66160d52d280d5225ea0782dbe40d997c56f30e924f1aeb13368f1",
            "483fdda3f143cdadeb28029cf4a95b9443a0f8eca9684aed31971c983e9d7f71",
            "4a60f26bbb13ccc401f6280b907607850f7ffac9199e2758f2e814af75481c9b",
        ];

        let expected_depth = 3;

        for i in 7..=14 {
            match get_node_info_from_db(&dynamodb_client, MERKLE_TREE_TABLE, i).await {
                Ok((depth, offset, hash)) => {
                    // Asserting the depth, offset, and hash
                    assert_eq!(depth, expected_depth, "Depth does not match for node {}", i);
                    assert_eq!(
                        offset,
                        i as usize - 7,
                        "Offset does not match for node {}",
                        i
                    );
                    assert_eq!(
                        hash,
                        expected_hash_sorted[i as usize - 7],
                        "Hash does not match for node {}",
                        i
                    );

                    println!(
                        "Node info - Depth: {}, Offset: {}, Hash: {}",
                        depth, offset, hash
                    );
                }
                Err(e) => {
                    eprintln!("Error fetching node info for node {}: {:?}", i, e);
                    panic!("Test failed due to error fetching node info");
                }
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
