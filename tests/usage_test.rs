use merkle_tree::{MerkleProof, MerkleTree};

#[test]
fn usage_test() {
    // A Merkle Tree can be built out of an array.
    let tree = MerkleTree::build(&[1, 2, 3]);

    // A Merkle Tree can generate a proof that it contains an element.
    let proof: MerkleProof = tree.get_proof(1);

    // A Merkle Tree can verify that a given hash is contained in it.
    let contains: bool = proof.verify(2); // Does the tree contain number 2?
    assert!(contains);

    // It is possible to create an empty tree.
    let mut empty_tree = MerkleTree::build::<u8>(&[]);

    // A Merkle Tree can be dynamic, this means that elements can be added once it is built.
    empty_tree.push(4);
    assert!(empty_tree.get_proof(0).verify(4));
}
