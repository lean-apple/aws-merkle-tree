use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct MerkleNode {
    index: u32,
    hash: String,
}

// impl fmt::Display for MerkleTreeError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "MerkleTree Error: {}", self.message)
//     }
// }

// impl Error for MerkleTreeError {}

impl MerkleNode {
    // Creates a new MerkleNode given an index and a value like potential leaf value
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

    async fn save_to_db(&self, client: &Client, table_name: &str) -> Result<(), Error> {
        // Store new MerkleNode in DynamoDB table

        let dynamo_item = self.clone().into_dynamodb_item();

        let set_request = client
            .put_item()
            .table_name(table_name)
            .set_item(Some(dynamo_item))
            .send()
            .await?;

        println!("set_request {:?}", set_request);
        Ok(())
    }
}

// Create and store the Merkle tree to DynamoDB
pub async fn create_and_store_merkle_tree(
    client: &Client,
    table_name: &str,
    leaves: &[&str],
) -> Result<(), Error> {
    let mut nodes = Vec::new();

    // Create leaves
    for (i, &leaf) in leaves.iter().enumerate() {
        nodes.push(MerkleNode::new(i as u32, leaf));
    }

    // Create internal nodes
    let mut i = 0;
    let mut next_index = leaves.len();
    while next_index - i > 1 {
        let parent_hash = {
            let left_child = &nodes[i];
            let right_child = &nodes[i + 1];

            let left_child_bytes =
                hex::decode(&left_child.hash).expect("Invalid hex in left child hash");
            let mut right_child_bytes =
                hex::decode(&right_child.hash).expect("Invalid hex in right child hash");

            let mut combined = left_child_bytes;
            combined.append(&mut right_child_bytes);

            //hasher.update(&combined);
            hex::encode(Sha3_256::digest(&combined))
        };

        let parent_node = MerkleNode {
            index: next_index as u32,
            hash: parent_hash,
        };
        // Store new parent node in DB
        parent_node.save_to_db(client, table_name).await?;
        nodes.push(parent_node);

        i += 2;
        next_index += 1;
    }

    Ok(())
}

pub fn is_valid_merkle_tree(nodes: &[MerkleNode]) -> bool {
    // Assuming the last node is the root
    let root = nodes.last().unwrap();
    is_valid_node(root, nodes)
}

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

    // Recursively check children
    is_valid_node(left_child, nodes) && is_valid_node(right_child, nodes)
}

pub async fn fetch_merkle_tree_from_db(
    client: &Client,
    table_name: &str,
) -> Result<Vec<MerkleNode>, Error> {
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
