use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MerkleNode {
    pub index: u32,
    pub hash: String, // Encode Hash
}

impl MerkleNode {
    // Create a new Merkl node given an index and a value like potential leaf datum
    pub fn new(index: u32, value: &str) -> Self {
        let hash = hex::encode(Sha3_256::digest(value.as_bytes()));
        MerkleNode { index, hash }
    }

    /// Create a parent node from the left and right children hashes
    pub fn to_parent_node(left: &MerkleNode, right: &MerkleNode, index: u32) -> Self {
        let left_hash = hex::decode(&left.hash).expect("Invalid hex in left child hash");
        let right_hash = hex::decode(&right.hash).expect("Invalid hex in left child hash");

        // Hash the concatenation of the values of the two child nodes
        let combined = [&left_hash[..], &right_hash[..]].concat();
        let combined_hash = Sha3_256::digest(&combined);

        let parent_hash = hex::encode(combined_hash);

        MerkleNode {
            index,
            hash: parent_hash,
        }
    }
}

// Create the entire Merkle tree and return all the nodes
pub fn create_tree(leaves: &[&str]) -> Result<Vec<MerkleNode>, Box<dyn std::error::Error>> {
    let mut nodes = Vec::new();

    // Create leaf nodes
    for (i, &leaf) in leaves.iter().enumerate() {
        nodes.push(MerkleNode::new(i as u32, leaf));
    }

    // Complete the leaf level to a power of two
    let next_power_of_two = leaves.len().next_power_of_two();
    while nodes.len() < next_power_of_two {
        let last = nodes.last().unwrap().clone();
        nodes.push(MerkleNode::new(nodes.len() as u32, &last.hash));
    }

    // Build the tree from the leaves up to the root
    let mut current_level_size = next_power_of_two;
    let mut level_start = 0;
    while current_level_size > 1 {
        for i in (0..current_level_size).step_by(2) {
            let left_child = &nodes[level_start + i];
            let right_child = &nodes[level_start + i + 1];

            let parent_node =
                MerkleNode::to_parent_node(left_child, right_child, nodes.len() as u32);
            nodes.push(parent_node);
        }
        level_start += current_level_size;
        current_level_size /= 2;
    }

    // Reverse nodes to put the root at index 0 and reassign indices
    nodes.reverse();
    for (new_index, node) in nodes.iter_mut().enumerate() {
        node.index = new_index as u32;
    }

    // Check the total number of nodes
    let expected_total_nodes = 2 * leaves.len() - 1;
    if nodes.len() != expected_total_nodes {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "The total number of nodes does not match the expected count",
        )));
    }

    // Return all nodes
    Ok(nodes)
}

/// Helper to check a merkle tree and its nodes
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
