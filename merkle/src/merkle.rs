use aws_sdk_dynamodb::{types::AttributeValue, Client, Error as DynamoError};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::error::Error as BasicError;

#[derive(Serialize, Deserialize, Clone)]
pub struct MerkleNode {
    index: u32,
    hash: String,
}

impl MerkleNode {
    // Creates a new MerkleNode given an index and a value like potential leaf datum
    fn new(index: u32, value: &str) -> Self {
        let hash = hex::encode(Sha3_256::digest(value.as_bytes()));
        MerkleNode { index, hash }
    }
    // Convert MerkleNode into AWS DynamoDB item
    fn into_dynamodb_item(self) -> HashMap<String, AttributeValue> {
        let mut item = HashMap::new();

        item.insert(
            "index".to_string(),
            AttributeValue::N(self.index.to_string()),
        );
        item.insert("hash".to_string(), AttributeValue::S(self.hash));

        item
    }

    // Store new MerkleTree Node in DynamoDB table
    async fn save_to_db(&self, client: &Client, table_name: &str) -> Result<(), DynamoError> {
        let dynamo_item = self.clone().into_dynamodb_item();

        match client
            .put_item()
            .table_name(table_name)
            .set_item(Some(dynamo_item))
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error saving to DB: {}", e); // Log the error
                Err(e.into())
            }
        }
    }
}

/// Create and store the Merkle tree to DynamoDB
pub async fn create_and_store_merkle_tree(
    client: &Client,
    table_name: &str,
    leaves: &[&str],
) -> Result<(), Box<dyn BasicError>> {
    // We assume we are in a total binary tree with a number of two children for every node
    if !is_power_of_two(leaves.len()) {
        let err = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "The number of leaves must be a power of two.",
        );
        return Err(Box::new(err));
    }

    let mut nodes = Vec::new();

    // Create leaves
    for (i, &leaf) in leaves.iter().enumerate() {
        nodes.push(MerkleNode::new(i as u32, leaf));
    }

    // Build the rest of the tree
    let mut current_level_nodes = leaves.len();
    while current_level_nodes > 1 {
        let previous_level_start = nodes.len() - current_level_nodes;
        for i in (0..current_level_nodes).step_by(2) {
            let left_child = &nodes[previous_level_start + i];
            let right_child = &nodes[previous_level_start + i + 1];

            let combined_hash = {
                let mut hasher = Sha3_256::new();
                hasher.update(
                    &hex::decode(&left_child.hash).expect("Invalid hex in left child hash"),
                );
                hasher.update(
                    &hex::decode(&right_child.hash).expect("Invalid hex in right child hash"),
                );
                hex::encode(hasher.finalize())
            };

            let parent_node = MerkleNode {
                index: (nodes.len() + 1) as u32, // Indexing for parent nodes
                hash: combined_hash,
            };

            nodes.push(parent_node);
        }
        current_level_nodes /= 2;
    }
    // Check the total number of nodes at the end
    let expected_total_nodes = 2 * leaves.len() - 1;
    if nodes.len() != expected_total_nodes {
        let err = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "The total number of nodes does not match the expected count.",
        );
        return Err(Box::new(err));
    }

    // Store all nodes in the DB
    for node in nodes {
        node.save_to_db(client, table_name).await?;
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

/// Helper to check merkle tree and its nodes
pub fn is_valid_merkle_tree(nodes: &[MerkleNode]) -> bool {
    // Assuming the last node is the root
    let root = nodes.last().unwrap();
    is_valid_node(root, nodes)
}

/// Helper to verify valid node
fn is_valid_node(node: &MerkleNode, nodes: &[MerkleNode]) -> bool {
    // Base case for leaf nodes
    if node.index >= nodes.len() as u32 / 2 {
        return true;
    }

    let left_index = node.index as usize * 2 + 1;
    let right_index = node.index as usize * 2 + 2;

    // Boundary checks
    if left_index >= nodes.len() || right_index >= nodes.len() {
        return false;
    }

    let left_child = &nodes[left_index];
    let right_child = &nodes[right_index];

    // Compute expected hash
    let expected_hash = {
        let mut hasher = Sha3_256::new();
        hasher.update(left_child.hash.as_bytes());
        hasher.update(right_child.hash.as_bytes());
        hex::encode(hasher.finalize())
    };

    // Check if current node's hash matches expected hash
    if node.hash != expected_hash {
        return false;
    }

    // Recursively check both children
    is_valid_node(left_child, nodes) && is_valid_node(right_child, nodes)
}

pub async fn fetch_merkle_tree_from_db(
    client: &Client,
    table_name: &str,
) -> Result<Vec<MerkleNode>, DynamoError> {
    let mut nodes = Vec::new();
    let mut last_evaluated_key = None;

    loop {
        let resp = client
            .scan()
            .table_name(table_name)
            .set_exclusive_start_key(last_evaluated_key)
            .send()
            .await?;

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

// Quick Helper to check the givne number is a power of two
fn is_power_of_two(n: usize) -> bool {
    n != 0 && (n & (n - 1)) == 0
}
