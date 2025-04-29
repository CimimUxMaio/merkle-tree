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

fn closest_power_of_2(number: usize) -> usize {
    (number as f32).log2().ceil().exp2() as usize
}

fn ancestor_index(index: usize, level: usize) -> usize {
    index / (2_usize.pow(level as u32)) // Integer division.
}

fn sibling_index(index: usize) -> usize {
    if index % 2 == 0 { index + 1 } else { index - 1 }
}

pub struct MerkleTree {
    pub levels: Vec<Vec<u64>>,
    pub padding: usize,
}

pub struct MerkleProof {
    pub index: usize,
    pub nodes: Vec<u64>,
    pub root: u64,
}

impl MerkleTree {
    pub fn build<T: Hash + Clone>(elements: Vec<T>) -> MerkleTree {
        if elements.is_empty() {
            panic!("Can not build a Merkle Tree with empty data");
        }

        let tree_width = closest_power_of_2(elements.len());
        let padding_len = tree_width - elements.len();
        let last = elements.last().expect("Empty data is not allowed").clone();
        let padding = vec![last; padding_len];
        let padded_elements: Vec<T> = elements.into_iter().chain(padding).collect();

        let mut levels = Vec::new();

        // Level 0 hashes
        let mut current: Vec<u64> = padded_elements.iter().map(hash_single).collect();
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

        MerkleTree {
            levels,
            padding: padding_len,
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
        MerkleTree::build(vec![1; 1]);
        MerkleTree::build(vec![1; 4]);
        MerkleTree::build(vec![1; 8]);
    }

    #[test]
    fn build_with_padded_trees() {
        MerkleTree::build(vec![1; 3]);
        MerkleTree::build(vec![1; 7]);
        MerkleTree::build(vec![1; 13]);
    }

    #[test]
    #[should_panic]
    fn build_with_empty_vector() {
        MerkleTree::build::<u8>(Vec::new());
    }

    #[test]
    fn height_of_tree() {
        let mut tests = Vec::new();

        // Power of two inputs.
        tests.push((MerkleTree::build(vec![1; 1]), 1));
        tests.push((MerkleTree::build(vec![1; 4]), 3));
        tests.push((MerkleTree::build(vec![1; 32]), 6));

        // Non power of two inputs.
        tests.push((MerkleTree::build(vec![1; 3]), 3));
        tests.push((MerkleTree::build(vec![1; 7]), 4));
        tests.push((MerkleTree::build(vec![1; 13]), 5));

        for (tree, expected_height) in tests {
            assert_eq!(tree.height(), expected_height);
        }
    }

    #[test]
    fn get_proof_from_populated_tree() {
        // Should not fail
        MerkleTree::build(vec![1; 1]).get_proof(0);
        MerkleTree::build(vec![1; 4]).get_proof(3);
        MerkleTree::build(vec![1; 8]).get_proof(4);

        MerkleTree::build(vec![1; 3]).get_proof(2);
        MerkleTree::build(vec![1; 7]).get_proof(7);
        MerkleTree::build(vec![1; 13]).get_proof(15);
    }

    #[test]
    fn proof_verifies() {
        let tree = MerkleTree::build(vec![1, 2, 3, 4]);
        assert!(tree.get_proof(2).verify(3));

        let tree = MerkleTree::build(vec![1, 2]);
        assert!(tree.get_proof(1).verify(2));

        let tree = MerkleTree::build(vec![1, 2, 3, 4, 5]);
        assert!(tree.get_proof(4).verify(5));
    }

    #[test]
    fn proof_not_verifies() {
        let tree = MerkleTree::build(vec![1, 2, 3, 4]);
        assert!(!tree.get_proof(3).verify(2));

        let tree = MerkleTree::build(vec![1, 2]);
        assert!(!tree.get_proof(1).verify(1));

        let tree = MerkleTree::build(vec![1, 2, 3, 4, 5]);
        assert!(!tree.get_proof(4).verify(4));
    }
}
