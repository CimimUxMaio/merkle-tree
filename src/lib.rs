use std::hash::{DefaultHasher, Hash, Hasher};

fn hash_single<H: Hash>(value: H) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn hash_pair<H: Hash>(first: H, second: H) -> u64 {
    let mut hasher = DefaultHasher::new();
    first.hash(&mut hasher);
    second.hash(&mut hasher);
    hasher.finish()
}

fn ancestor_index(index: usize, level: usize) -> usize {
    index / (2_usize.pow(level as u32)) // Integer division.
}

fn sibling_index(index: usize) -> usize {
    if index % 2 == 0 { index + 1 } else { index - 1 }
}

pub struct MerkleTree {
    pub levels: Vec<Vec<u64>>,
    capacity: usize,
    padding: usize,
}

pub struct MerkleProof {
    index: usize,
    nodes: Vec<u64>,
    root: u64,
}

fn generate_levels(leafs: &Vec<u64>, levels: &mut Vec<Vec<u64>>) {
    let mut current: Vec<u64> = leafs.to_owned();
    levels.push(current.clone());

    while current.len() > 1 {
        let current_len = current.len();
        let mut next_level = Vec::new();
        for index in (0..current_len).step_by(2) {
            let hash = hash_pair(current[index], current[index + 1]);
            next_level.push(hash);
        }
        current = next_level;
        levels.push(current.clone());
    }
}

impl MerkleTree {
    fn pad() -> u64 {
        0
    }

    pub fn build<H: Hash>(elements: &[H]) -> MerkleTree {
        if elements.is_empty() {
            panic!("Can not build a Merkle Tree with empty data");
        }

        let capacity = elements.len().next_power_of_two();
        let padding = capacity - elements.len();
        let padding_vec = vec![MerkleTree::pad(); padding];

        // Level 0 hashes
        let leafs = elements
            .iter()
            .map(hash_single)
            .chain(padding_vec)
            .collect();

        let mut levels = Vec::new();
        generate_levels(&leafs, &mut levels);

        MerkleTree {
            levels,
            capacity,
            padding,
        }
    }

    pub fn height(&self) -> usize {
        self.levels.len()
    }

    pub fn get_proof(&self, index: usize) -> MerkleProof {
        let mut nodes: Vec<u64> = Vec::new();

        for level_n in 0..self.levels.len() - 1 {
            let ancestor = ancestor_index(index, level_n);
            let proof_node_index = sibling_index(ancestor);
            nodes.push(self.levels[level_n][proof_node_index]);
        }

        MerkleProof {
            nodes,
            index,
            root: self.root(),
        }
    }

    pub fn root(&self) -> u64 {
        self.levels[self.height() - 1][0]
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        match self.levels.first() {
            Option::None => 0_usize,
            Option::Some(level) => level.len() - self.padding,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_full(&self) -> bool {
        self.padding == 0
    }

    fn duplicate_capacity(&mut self) {
        // Generate new nodes.
        let new_leafs = vec![MerkleTree::pad(); self.capacity];
        let mut new_levels = Vec::new();
        generate_levels(&new_leafs, &mut new_levels);

        // Append new nodes to each level of the tree.
        for (level_n, level) in new_levels.iter_mut().enumerate() {
            self.levels[level_n].append(level);
        }

        // Re-compute root node;
        let last_level = &self.levels[self.height() - 1];
        let new_root = hash_pair(last_level[0], last_level[1]);
        self.levels.push(vec![new_root]);

        // Update padding;
        self.padding += self.capacity;

        // Update capacity;
        self.capacity *= 2;
    }

    pub fn push<H: Hash>(&mut self, value: H) {
        if self.is_full() {
            self.duplicate_capacity();
        }

        let mut index = self.len();
        self.levels[0][index] = hash_single(value);

        for level_n in 1..self.levels.len() {
            let previous_level = &self.levels[level_n - 1];
            let node = previous_level[index];
            let sibling_node = previous_level[sibling_index(index)]; // Previous index's sibling.

            let parent_index = ancestor_index(index, 1);

            self.levels[level_n][parent_index] = if index % 2 == 0 {
                hash_pair(node, sibling_node)
            } else {
                hash_pair(sibling_node, node)
            };

            index = parent_index;
        }

        self.padding -= 1;
    }
}

impl MerkleProof {
    pub fn verify<H: Hash>(&self, value: H) -> bool {
        let mut computed_root = hash_single(value);

        for node_n in 0..self.nodes.len() {
            let ancestor = ancestor_index(self.index, node_n);

            computed_root = if ancestor % 2 == 0 {
                hash_pair(computed_root, self.nodes[node_n])
            } else {
                hash_pair(self.nodes[node_n], computed_root)
            };
        }

        computed_root == self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_with_power_of_2_elements() {
        MerkleTree::build(&[1; 1]);
        MerkleTree::build(&[1; 4]);
        MerkleTree::build(&[1; 8]);
    }

    #[test]
    fn build_with_padded_trees() {
        MerkleTree::build(&[1; 3]);
        MerkleTree::build(&[1; 7]);
        MerkleTree::build(&[1; 13]);
    }

    #[test]
    #[should_panic]
    fn build_with_empty_array() {
        MerkleTree::build::<u8>(&[]);
    }

    #[test]
    fn height_of_tree() {
        let mut tests = Vec::new();

        // Power of two inputs.
        tests.push((MerkleTree::build(&[1; 1]), 1));
        tests.push((MerkleTree::build(&[1; 4]), 3));
        tests.push((MerkleTree::build(&[1; 32]), 6));

        // Non power of two inputs.
        tests.push((MerkleTree::build(&[1; 3]), 3));
        tests.push((MerkleTree::build(&[1; 7]), 4));
        tests.push((MerkleTree::build(&[1; 13]), 5));

        for (tree, expected_height) in tests {
            assert_eq!(tree.height(), expected_height);
        }
    }

    #[test]
    fn get_proof_from_populated_tree() {
        // Should not fail
        MerkleTree::build(&[1; 1]).get_proof(0);
        MerkleTree::build(&[1; 4]).get_proof(3);
        MerkleTree::build(&[1; 8]).get_proof(4);

        MerkleTree::build(&[1; 3]).get_proof(2);
        MerkleTree::build(&[1; 7]).get_proof(7);
        MerkleTree::build(&[1; 13]).get_proof(15);
    }

    #[test]
    fn proof_verifies() {
        let tree = MerkleTree::build(&[1, 2, 3, 4]);
        assert!(tree.get_proof(2).verify(3));

        let tree = MerkleTree::build(&[1, 2]);
        assert!(tree.get_proof(1).verify(2));

        let tree = MerkleTree::build(&[1, 2, 3, 4, 5]);
        assert!(tree.get_proof(4).verify(5));
    }

    #[test]
    fn proof_not_verifies() {
        let tree = MerkleTree::build(&[1, 2, 3, 4]);
        assert!(!tree.get_proof(3).verify(2));

        let tree = MerkleTree::build(&[1, 2]);
        assert!(!tree.get_proof(1).verify(1));

        let tree = MerkleTree::build(&[1, 2, 3, 4, 5]);
        assert!(!tree.get_proof(4).verify(4));
    }

    #[test]
    fn push_value_with_capacity() {
        let mut tree = MerkleTree::build(&[1, 2, 3]);
        assert!(!tree.get_proof(3).verify(4));
        tree.push(4);
        assert!(tree.get_proof(3).verify(4));

        let mut tree = MerkleTree::build(&[1; 6]);
        assert!(!tree.get_proof(6).verify(2));
        tree.push(2);
        assert!(tree.get_proof(6).verify(2));
        assert!(!tree.get_proof(7).verify(3));
        tree.push(3);
        assert!(tree.get_proof(7).verify(3));
    }

    #[test]
    fn push_value_without_capacity() {
        let mut tree = MerkleTree::build(&[1, 2]);
        tree.push(3);
        tree.get_proof(2).verify(3);

        let mut tree = MerkleTree::build(&[1; 8]);
        tree.push(2);
        tree.push(3);
        tree.get_proof(9).verify(3);
    }
}
