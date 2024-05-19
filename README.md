# pyo3Tree

Rust based Tree datastructure wrapped by pyo3.

## Usage

You can either clone the repo and build:

```
pip install maturin
cd tree_py
maturin develop # Does the magic of module building
maturin develop --release # Build an optimized version
```

Or install it with pip:

```
pip install pyo3Tree
```

Then to use:

```
from pyo3Tree import Tree
from pyo3Tree import Node

tree = Tree()
child = Node('child')
grand_child = Node('grand child')

tree.add_child(child)
tree.add_child(grand_child,child)
```

## Current functions:

### Node
- Node(data) - returns a Python owned reference to a rust owned Node containing a reference to the python object passed in as data.
- Node has four attributes: id, data, children[], parent. 
- node.id - returns a python owned string with the nodes uuid
- node.data - returns a python owned reference to the python owned object stored as data
- node.children - returns a python owned vector containing python owned references to the rust owned children nodes
- node.parent - returns a python owned reference to the rust owned parent node 

### Tree
- Tree() - builds an empty tree with a root node containing no data
- Tree(node) - builds a tree with the specified node as root
- Tree.import(pythonDictionary) - imports the specified python tree (which should be of type PyDict) and returns a reference to the rust Tree.
- tree.find_by_id("uuid") - returns a python owned reference to the rust owned Node with id "uuid"
- tree.add(node) - Adds the node to the trees root node
- tree.add(node, parentNode) - adds the node as a child of the parent node
- tree.get_root() - returns a python owned reference to the rust owned root node
- tree.move_node(tgt_node, parent_node) - moves the tgt_node to the parent node. This throws an error if the parent node is an ancestor of the child node. Note: 'move' is a reserved word in rust and functions cannot be named 'move'
- tree.get_ancestors(node) - returns a python owned vector of python owned references to the rust owned ancestors of the specified node.
- tree.export() - returns a completely python owned dictionary representation of the Tree.

### NOTES: 
- Constructing a tree creates a python object containing a reference to the rust object. The nodes can hold any Python object (which will be tracked by Pythons memory mananger). The rest of the tree should be managed by Rust, on a combination of the stack and the heap. If I understand it correctly, each node and tree instance will exist on the stack, whereas all the Vectors and reference counters will live on the heap.
- the file tree_py.rs is a wrapper of tree_rs.rs and provides the interfaces to the rust objects and their attributes.
- Node and Tree are compatible with threading (pyo3 wasn't allowing the rust single thread reference implementation).
- The rust implementation uses 'Atomic Reference counters' to determine whether to drop objects, each python owned reference to a rust owned Node or Tree adds to the reference counter.
- Each piece of data stored is fully owned by Python, the rust implementation stores a reference to the Python object, which I presumes adds to Pythons reference count for that object.

This is roughly as fast as the python tree implementations, however 'find_node_by_id' is much faster ~10x. It seems like the initial object generation is roughly the same speed as the python implementations bigtree, anytree.

No large tree testing has occurred. No performance or memory optimization has occurred.