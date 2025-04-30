# Merkle Tree
A Merkle Tree implementation written in Rust.

# Usage
```rust
use merkle_tree::{MerkleTree, MerkleProof};

// A Merkle Tree can be built out of an array.
let tree = MerkleTree::build(&[1, 2, 3]);

// A Merkle Tree can generate a proof that it contains an element.
let proof: MerkleProof = tree.get_proof(1);

// A Merkle Tree can verify that a given hash is contained in it.
let contains: bool = proof.verify(2); // Does the tree contain number 2?

// It is possible to create an empty tree.
let empty_tree = MerkleTree::build::<u8>(&[]);

// A Merkle Tree can be dynamic, this means that elements can be added once it is built.
empty_tree.push(4);
```

- You can run `make docs` to check the full documentation.

# How it Works
Leaf nodes are generated with the hashes of each input element. Then, parent nodes are computed
as the hash between both of their child nodes until reaching the root.

To be able to handle any amount of elements, trees have an allocated capacity which will always
be a power of 2. This means that if the amount of elements from which it is being generated
is not a power of 2, aditional "padding" values will be included until reaching the tree's capacity.

When a new element is pushed, if there is enough capacity to allocate it, the new value's hash will
replace the first "padding" leaf value and will trigger the update of only the parents in the
corresponding section of the tree.

If there is not enough capacity, the tree's capacity will be automatically doubled in order to
allocate the new value.

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
