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

pub struct MerkleTree {
    pub levels: Vec<Vec<u64>>,
    pub padding: usize,
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
}
