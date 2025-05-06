use std::hash::{DefaultHasher, Hash, Hasher};

/// Returns the hash of a single value. The value's type must implement
/// the `Hash` trait.
fn hash_single<H: Hash>(value: H) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Returns the hash resulting of combining two values.
/// Both values must implement the `Hash` trait.
fn hash_pair<H: Hash>(first: H, second: H) -> u64 {
    let mut hasher = DefaultHasher::new();
    first.hash(&mut hasher);
    second.hash(&mut hasher);
    hasher.finish()
}

/// Given an node's index. Returns the index of the ancestor in the given level,
/// relative to the child node's level.
/// Each level of the tree can be thought as a vector of nodes, in this context, an index for
/// a given level represents the index within this vector.
/// * `index` - The child index.
/// * `level` - The relative upwards level of the target ancestor.
fn ancestor_index(index: usize, level: usize) -> usize {
    index / (2_usize.pow(level as u32)) // Integer division.
}

/// Given a node's index. Returns the index of its sibling node.
/// Each level of the tree can be thought as a vector of nodes, in this context, an index for
/// a given level represents the index within this vector.
/// The sibling node is the node on the same level that shares the same parent.
/// * `index` - The target node's index.
fn sibling_index(index: usize) -> usize {
    if index % 2 == 0 { index + 1 } else { index - 1 }
}

/// Given the leaves of a tree (the first level of the tree), generates all
/// its upper levels (ancestors) by computing the hashes of each pair iteratively.
/// * `leaves` - Level 0, the starting leaves.
/// * `levels` - Vector where the generated levels will be stored.
fn generate_tree_levels(leaves: &Vec<u64>, levels: &mut Vec<Vec<u64>>) {
    let mut current: Vec<u64> = leaves.to_owned();
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

/// Base structure were merkle tree data is stored.
pub struct MerkleTree {
    levels: Vec<Vec<u64>>,
    capacity: usize,
    padding: usize,
}

/// Contains merkle proof information for later validation.
pub enum MerkleProof {
    Proof {
        index: usize,
        nodes: Vec<u64>,
        root: u64,
    },

    /// Invalid proofs always return false for `proof.verify(value)`.
    Invalid,
}

impl MerkleTree {
    /// The default `Hash` value that is used as padding.
    const PAD_HASH: u64 = 0;

    /// Constructs a `MerkleTree` and populates it with the provided elements as leaf nodes.
    /// Each leaf node is hashed and stored in the frist level of the tree. Then, each
    /// pair of nodes is used to compute their parent by hashing both of its values (hashes) until reaching
    /// the root node.
    /// * `elements` - array of `Hash` elements used to populate the tree.
    pub fn build<H: Hash>(elements: &[H]) -> MerkleTree {
        let capacity = elements.len().next_power_of_two();
        let padding = capacity - elements.len();
        let padding_vec = vec![MerkleTree::PAD_HASH; padding];

        // Level 0 hashes
        let leaves = elements
            .iter()
            .map(hash_single)
            .chain(padding_vec)
            .collect();

        let mut levels = Vec::new();
        generate_tree_levels(&leaves, &mut levels);

        MerkleTree {
            levels,
            capacity,
            padding,
        }
    }

    /// Returns the height of the tree.
    pub fn height(&self) -> usize {
        self.levels.len()
    }

    /// Creates a `MerkleProof` for a given index.
    /// Attempting to create a proof for an invalid node (i.e. using an index which
    /// does not correspond to a valid leaf) will return a `MerkleProof::Invalid`
    /// value.
    /// * `index` - index value to generate the proof for.
    pub fn get_proof(&self, index: usize) -> MerkleProof {
        let is_invalid_index = index >= self.len();
        if is_invalid_index || self.is_empty() {
            return MerkleProof::Invalid;
        }

        let mut nodes: Vec<u64> = Vec::new();

        for level_n in 0..self.levels.len() - 1 {
            let ancestor = ancestor_index(index, level_n);
            let proof_node_index = sibling_index(ancestor);
            nodes.push(self.levels[level_n][proof_node_index]);
        }

        MerkleProof::Proof {
            nodes,
            index,
            root: self.root().expect("Non-empty trees always have a root"),
        }
    }

    /// Returns the root of the tree. If the tree is empty, the root will be `None`.
    pub fn root(&self) -> Option<u64> {
        if self.is_empty() {
            return None;
        }
        self.levels.get(self.height() - 1)?.first().copied()
    }

    /// Returns the capacity of the tree.
    /// The capacity is the amount of space it has allocated.
    /// It may be different from the tree's length.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the length of the tree.
    /// The length of the tree is the amount of elements it contains.
    /// It may be different from the tree's capacity.
    pub fn len(&self) -> usize {
        match self.levels.first() {
            Option::None => 0_usize,
            Option::Some(level) => level.len() - self.padding,
        }
    }

    /// Returns wether a tree has no elements or not.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns wether a tree is at full capacity or not.
    pub fn is_full(&self) -> bool {
        self.padding == 0
    }

    /// Duplicates the capacity of a given tree.
    /// This involves creating a new root node where one of its children will
    /// be the current root node, and the other, will be the root node of a new
    /// subtree of the same height of the current, filled with padding values.
    /// This operation also results in the tree increasing its height by 1 level.
    fn duplicate_capacity(&mut self) {
        // Generate new nodes.
        let new_leaves = vec![MerkleTree::PAD_HASH; self.capacity];
        let mut new_levels = Vec::new();
        generate_tree_levels(&new_leaves, &mut new_levels);

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

    /// Pushes an `Hash` element into the tree.
    /// This will only trigger the update of new node's ancestors.
    /// If the tree does not have enough capacity, more space will be
    /// allocated and its capacity will be doubled.
    /// * `value` - The `Hash` value to be added to the tree.
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
    /// Returns whether a given `Hash` value verifies the proof.
    /// * `value` - The `Hash` value to be tested.
    pub fn verify<H: Hash>(&self, value: H) -> bool {
        match self {
            MerkleProof::Invalid => false,
            MerkleProof::Proof { index, nodes, root } => {
                let mut computed_root = hash_single(value);

                for (node_n, &node) in nodes.iter().enumerate() {
                    let ancestor = ancestor_index(*index, node_n);

                    computed_root = if ancestor % 2 == 0 {
                        hash_pair(computed_root, node)
                    } else {
                        hash_pair(node, computed_root)
                    };
                }

                computed_root == *root
            }
        }
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
    fn get_proof_from_empty_tree() {
        MerkleTree::build::<u8>(&[]).get_proof(10);
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

        // Should return false if the tree is empty.
        let tree = MerkleTree::build::<u8>(&[]);
        assert!(!tree.get_proof(10).verify(2));

        // Should return false for an invalid index.
        let tree = MerkleTree::build(&[1, 2, 3]);
        assert!(!tree.get_proof(10).verify(2));
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
