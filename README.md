# Merkle Tree
A Merkle Tree implementation written in Rust.

# Usage
```rust
use merkle_tree::{MerkleTree, MerkleProof};

// A Merkle Tree can be built out of an array.
let tree = MerkleTree::build(&[1, 2, 3]);

// A Merkle Tree can generate a proof that it contains an element at a given leaf index.
let proof: MerkleProof = tree.get_proof(1);

// A Merkle Proof can verify that a given hash is contained in it.
let contains = proof.verify(2); // Does the tree contain number 2 at leaf 1?
println!("{}", contains); // Prints: true

// It is possible to create an empty tree.
let mut empty_tree = MerkleTree::build::<u8>(&[]);

// A Merkle Tree can be dynamic, this means that elements can be added once it is built.
empty_tree.push(4);
```

- You can run `make docs` to check the full documentation.

# How it Works

A Merkle Tree is a tree in which every leaf is labelled with the cryptographic hash of a data block,
and every node that is not a leaf is labeled with the cryptographic hash of the labels of its
child nodes.

Merkle Trees allows efficient and secure verification of contents of large data structures, since
it only stores hashes and not full files / blocks of data.

In this implementation, a Merkle Tree consists of a binary tree, and can be generated given
an array of hashable elements.

Each leaf node corresponds to the hash of an element from the input array (in order). Then, each
pair of nodes is used to compute their parent by hashing both of its values (hashes) until reaching
the root node.

To be able to store any arbitrary amount of elements, trees have an allocated capacity which
will always be a power of 2. This means that if the amount of elements from which it is being generated
(input elements) is not a power of 2, additional "padding" values will be included until reaching the
tree's capacity.

When a new element is pushed into the tree, if there is enough capacity to allocate it, the new value's
hash will replace the first "padding" leaf value and will only trigger the update of its ancestors.

If there is not enough capacity, it will be automatically doubled in order to allocate the new value.

# Useful Commands
- To build the project.
```
make
```
or
```
make build
```
- To run tests.
```
make test
```
- To open the documentation.
```
make docs
```
- To clean the current build.
```
make clean
```

# Dependencies
- rust = "1.86.0"
