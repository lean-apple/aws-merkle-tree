use crate::merkle::{create_tree, MerkleNode};
use aws_sdk_dynamodb::{types::AttributeValue, Client, Error as DynamoError};
use std::collections::HashMap;
use std::error::Error as BasicError;

// Helper to fetch all merkle tree nodes in a sorted way
// From DynamoDB
pub async fn fetch_merkle_tree_from_db(
    client: &Client,
    table_name: &str,
) -> Result<Vec<MerkleNode>, DynamoError> {
    let mut nodes = Vec::new();
    let mut last_evaluated_key = None;

    loop {
        let resp = match client
            .scan()
            .table_name(table_name)
            .set_exclusive_start_key(last_evaluated_key)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(err) => return Err(DynamoError::from(err)),
        };

        if let Some(items) = resp.items {
            for item in items {
                let index = match item.get("index") {
                    Some(v) => v.as_n().unwrap().parse::<u32>().unwrap_or_default(),
                    None => 0,
                };
                let hash = match item.get("hash") {
                    Some(v) => v.as_s().unwrap().clone(),
                    None => String::new(),
                };

                nodes.push(MerkleNode { index, hash });
            }
        }
        if resp.last_evaluated_key.is_none() {
            break;
        }

        last_evaluated_key = resp.last_evaluated_key;
    }

    // Sort nodes by index
    nodes.sort_by(|a, b| a.index.cmp(&b.index));

    Ok(nodes)
}

// Store MerkleTree Node in DynamoDB table
async fn save_to_db(
    node: &MerkleNode,
    client: &Client,
    table_name: &str,
) -> Result<(), DynamoError> {
    let dynamo_item = into_dynamodb_item(node);

    match client
        .put_item()
        .table_name(table_name)
        .set_item(Some(dynamo_item))
        .send()
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error saving to DB: {}", e);
            Err(e.into())
        }
    }
}

/// Create and store ALL the Merkle tree to DynamoDB
/// Generate the merkle tree from the leaves data argument
pub async fn create_and_store_merkle_tree(
    client: &Client,
    table_name: &str,
    leaves: &[&str],
) -> Result<(), Box<dyn BasicError>> {
    let tree_nodes = create_tree(leaves).expect("Failed to build Merkle tree from given leaves");

    // Store all nodes in the DB ie. all the new tree
    for node in tree_nodes {
        save_to_db(&node, client, table_name).await?;
    }

    Ok(())
}

// Get node info ie (depth, offset, leave value) in DynamoDB from node index
pub async fn get_node_info_from_db(
    client: &Client,
    table_name: &str,
    index: u32,
) -> Result<(usize, usize, String), Box<dyn BasicError>> {
    let resp = client
        .get_item()
        .table_name(table_name)
        .key("index", AttributeValue::N(index.to_string()))
        .send()
        .await;

    match resp {
        Ok(response) => {
            if let Some(item) = response.item {
                let hash = item
                    .get("hash")
                    .and_then(|v| v.as_s().ok())
                    .cloned()
                    .unwrap_or_default();

                let mut depth: usize = 0;

                if index == 0 {
                    return Ok((0, 0, hash));
                }

                let mut nodes_numb_test: usize = 1;

                // While the possible number of nodes is below the node index
                // We haven't reached the final layer
                while nodes_numb_test <= index as usize {
                    depth += 1;
                    nodes_numb_test += 2usize.pow(depth as u32);
                }

                // We are looking for the number of nodes of the previous layer
                // to substract them from the index to get the offset
                let prev_nodes_layer_total = nodes_numb_test - 2_usize.pow(depth as u32);

                Ok((depth, index as usize - prev_nodes_layer_total, hash))
            } else {
                let err = std::io::Error::new(std::io::ErrorKind::NotFound, "Node not found");
                Err(Box::new(err))
            }
        }
        Err(e) => Err(Box::new(e)),
    }
}

// Helper to convert MerkleNode into AWS DynamoDB item
pub fn into_dynamodb_item(node: &MerkleNode) -> HashMap<String, AttributeValue> {
    let mut item = HashMap::new();

    item.insert(
        "index".to_string(),
        AttributeValue::N(node.index.to_string()),
    );
    item.insert("hash".to_string(), AttributeValue::S(node.clone().hash));

    item
}

// List and print available DynamoDB tables
pub async fn list_tables(client: &Client) -> Result<(), DynamoError> {
    let _resp = client.list_tables().send().await?;
    Ok(())
}
