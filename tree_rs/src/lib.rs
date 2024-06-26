use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock, Weak as AWeak};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use pyo3::{PyObject, Python, ToPyObject};

pub struct Tree {
    pub root: Arc<Mutex<Node>>,
}

pub struct TreeMap {
    pub nodes: Arc<RwLock<HashMap<String,Arc<RwLock<NodeMap>>>>>,
}

impl TreeMap {
    pub fn new(root: Option<Arc<RwLock<NodeMap>>>) -> Self {
        let nodes = Arc::new(RwLock::new(HashMap::with_capacity(100)));
        match root {
            Some(node) => {
                let node_id = node.read().unwrap().id.clone();
                let mut nodes_guard = nodes.write().unwrap();
                nodes_guard.insert(node_id, node.clone());
                nodes_guard.insert("root".to_string(),node)
            },
            None => {
                let node = NodeMap::new(None);
                let node_id = node.read().unwrap().id.clone();
                let mut nodes_guard = nodes.write().unwrap();
                nodes_guard.insert(node_id, node.clone());
                nodes_guard.insert("root".to_string(),node)
            }
        };
        Self {nodes}
    }

    pub fn add_child(&self, child: &Arc<RwLock<NodeMap>>, parent: Option<&Arc<RwLock<NodeMap>>>) -> Result<()> {
        let mut nodes_guard = self.nodes.write().unwrap();
        nodes_guard.insert(child.read().unwrap().id.clone(), child.clone());
        // TODO decide how to handle a parent not being on the tree
        // nodes_guard.insert(parent.unwrap().as_ref().read().unwrap().id.clone(), parent.unwrap().clone());

        // Checks for parent inside option, if no parent, make parent 'root node'
        let (parent_id, mut parent_guard) = match parent {
            Some(node) => {
                let parent_guard = node.write().unwrap();
                (parent_guard.id.clone(), parent_guard)
            }
            None => {
                ("root".to_string(), nodes_guard.get("root").unwrap().write().unwrap())
            }
        };
        let mut child_guard = child.write().unwrap();
        parent_guard.children.push(child_guard.id.clone());
        child_guard.parent = Some(parent_id);

        Ok(())
    }

    pub fn find_by_id(&self, id: &str) -> Result<Arc<RwLock<NodeMap>>> {
        Ok(self.nodes.read().unwrap().get(id).unwrap().clone())
    }

    pub fn get_ancestors(&self, node: &Arc<RwLock<NodeMap>>) -> Result<Vec<Arc<RwLock<NodeMap>>>> {
        let mut collection: Vec<Arc<RwLock<NodeMap>>> = Vec::with_capacity(50);
        get_nodemap_ancestors_recursive(self, &node, &mut collection);
        collection.shrink_to_fit();
        Ok(collection)
    }

// TODO - need this for when we start dropping NodeMaps
    // pub fn remove_from_parent(&self, node: &Arc<RwLock<NodeMap>>) {
    // }

    pub fn move_node(&self, tgt_node: &Arc<RwLock<NodeMap>>, new_parent: &Arc<RwLock<NodeMap>>) -> Result<()> {
        // If child is an ancestor of new_parent Error out
        if self.get_ancestors(new_parent).unwrap().iter().any(|node| Arc::ptr_eq(node, &tgt_node)) {
            Err(anyhow!("Input node is ancestor of parent, cannot move."))?
        }
        
        // obtain tgt_node write lock to pull out data and set new parent
        let mut tgt_node_guard = tgt_node.write().unwrap();
        let tgt_node_id = tgt_node_guard.id.clone();
        
        {  
            // obtain old_parent_write_guard, find the tgt_node id and remove it.
            let old_parent = self.nodes.read().unwrap().get(tgt_node_guard.parent.as_ref().unwrap()).unwrap().clone();
            let mut old_parent_write_guard = old_parent.write().unwrap();
            if let Some(index) = old_parent_write_guard.children.iter().position(|x| x == &tgt_node_id){
                old_parent_write_guard.children.remove(index);
            }
        }

        let mut new_parent_guard = new_parent.write().unwrap();
        new_parent_guard.children.push(tgt_node_id);
        tgt_node_guard.parent = Some(new_parent_guard.id.clone());
        drop(new_parent_guard);
        drop(tgt_node_guard);    

        Ok(())
    }
}

pub fn get_nodemap_ancestors_recursive(tree: &TreeMap, node: &Arc<RwLock<NodeMap>>, collection: &mut Vec<Arc<RwLock<NodeMap>>>) {
    if let Some(parent) = node.read().unwrap().parent.clone() {
        let parent_node = tree.nodes.read().unwrap().get(&parent).unwrap().clone();
        collection.push(parent_node.clone());
        get_nodemap_ancestors_recursive(tree, &parent_node, collection);
    }
}

impl Tree {
    pub fn new(root: Option<Arc<Mutex<Node>>>) -> Arc<Mutex<Self>> {
        match root {
            Some(node) => Arc::new(Mutex::new(Self {root: node})),
            None => Arc::new(Mutex::new(Self {root: Node::new(py_none(), None)}))
        }
    }

    pub fn add_child(&self, child: Arc<Mutex<Node>>, parent_node: Option<Arc<Mutex<Node>>>) {
        let parent: Arc<Mutex<Node>> = parent_node.unwrap_or_else(|| self.root.clone());
        {
            parent.lock().unwrap()
                .children.lock().unwrap()
                .push(Arc::clone(&child));
        }
        let self_weak: AWeak<Mutex<Node>> = Arc::downgrade(&parent);
        child.lock().unwrap().parent = Some(self_weak);
    }

    pub fn find_by_id(&self, id: &str) -> Option<Arc<Mutex<Node>>> {
        let mut stack = VecDeque::new();
        stack.push_back(self.root.clone());

        while let Some(node_arc) = stack.pop_front() {
            let node = node_arc.lock().unwrap();

            if node.id == id {
                return Some(node_arc.clone());
            }

            for child in node.children.lock().unwrap().iter(){
                stack.push_back(child.clone());
            }
        }

        None
    }

    pub fn get_ancestors(&self, node: &Arc<Mutex<Node>>) -> Vec<Arc<Mutex<Node>>> {
        let mut collection: Vec<Arc<Mutex<Node>>> = Vec::new();
        get_ancestors_recursive(node, &mut collection);
        collection
    }

    pub fn move_node(&self, tgt_node: &Arc<Mutex<Node>>, new_parent_node: &Arc<Mutex<Node>>) -> (){
        if self.get_ancestors(new_parent_node).iter().any(|ancestor| Arc::ptr_eq(ancestor, tgt_node)) {
            println!("Operation not allowed: Cannot move a node into one of its descendants.");
            return;
        }

        // Remove the node from its current parent's children list, if it has one
        if let Some(ref parent_weak) = tgt_node.lock().unwrap().parent {
            if let Some(parent) = parent_weak.upgrade() {
                {
                    let parent_borrowed = parent.lock().unwrap();
                    let mut parent_children = parent_borrowed.children.lock().unwrap();
                    if let Some(index) = parent_children.iter().position(|child| Arc::ptr_eq(child, tgt_node)) {
                        parent_children.remove(index);
                    }
                }
            }
        }

        // Add the node to the new parent's children list
        new_parent_node.lock().unwrap().children.lock().unwrap().push(Arc::clone(tgt_node));

        // Update the parent reference in the target node
        let new_parent_weak = Arc::downgrade(new_parent_node);
        tgt_node.lock().unwrap().parent = Some(new_parent_weak);
    }
}

fn get_ancestors_recursive(node: &Arc<Mutex<Node>>, collection: &mut Vec<Arc<Mutex<Node>>>) {
    if let Some(parent_weak) = node.lock().unwrap().parent.as_ref() {
        if let Some(parent) = parent_weak.upgrade() {
            collection.push(parent.clone());
            get_ancestors_recursive(&parent, collection);
        }
    }
}

pub struct Node {
    pub id: String,
    pub data: PyObject,
    pub children: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
    // Option only to cater for 'root'
    pub parent: Option<AWeak<Mutex<Node>>>,
}

impl Node {
    pub fn new(data: PyObject, parent: Option<AWeak<Mutex<Node>>>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            id: Uuid::new_v4().to_string(),
            data,
            children: Arc::new(Mutex::new(Vec::new())),
            parent,
        }))
    }
}

pub struct NodeMap {
    pub id: String,
    pub children: Vec<String>,
    pub parent: Option<String>,
}

impl NodeMap {
    pub fn new(parent: Option<String>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new( Self {
            id: Uuid::new_v4().to_string(),
            children: Vec::with_capacity(5),
            parent
        }))
    }
}

pub fn py_none() -> PyObject {
    Python::with_gil(|py| {
        py.None().to_object(py)
    })
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add_node_to_empty_tree_mt(){
        let tree = Tree::new(None);
        let child_node = Node::new(py_none(), None);

        tree.lock().unwrap().add_child(child_node.clone(), None);

        let child_parent_id = child_node.lock().unwrap().parent.as_ref().unwrap().upgrade().unwrap().lock().unwrap().id.clone();

        let root_id = tree.lock().unwrap().root.lock().unwrap().id.clone();

        assert_eq!(child_parent_id, root_id);

        assert!(tree.lock().unwrap().root.lock().unwrap().children.lock().unwrap().iter().any(|child| Arc::ptr_eq(&child, &child_node)));
    }

    #[test]
    fn test_add_node_two_deep_mt(){
        let tree = Tree::new(None);
        let child_node = Node::new(py_none(), None);
        let childs_child_node = Node::new(py_none(), None);

        tree.lock().unwrap().add_child(child_node.clone(), None);
        tree.lock().unwrap().add_child(childs_child_node.clone(), Some(child_node.clone()));

        let childs_child_parent_id = childs_child_node.lock().unwrap().parent.as_ref().unwrap().upgrade().unwrap().lock().unwrap().id.clone();

        let child_id = child_node.lock().unwrap().id.clone();

        assert_eq!(childs_child_parent_id, child_id);

        assert!(child_node.lock().unwrap().children.lock().unwrap().iter().any(|child| Arc::ptr_eq(&child, &childs_child_node)));
    }

    #[test]
    fn test_find_by_id_mt(){
        let tree = Tree::new(None);
        let child_node = Node::new(py_none(), None);
        let childs_child_node = Node::new(py_none(), None);

        {
            tree.lock().unwrap().add_child(child_node.clone(), None);
            tree.lock().unwrap().add_child(childs_child_node.clone(), Some(child_node.clone()));
        }

        let childs_child_id =  {childs_child_node.lock().unwrap().id.clone()};
        let found_child = {tree.lock().unwrap().find_by_id(&childs_child_id).unwrap().clone()};
        assert!(Arc::ptr_eq(&found_child,&childs_child_node));
    }

    #[test]
    fn test_get_ancestors_on_two_deep_tree_mt(){
        let tree = Tree::new(None);
        let child_node = Node::new(py_none(), None);
        let childs_child_node = Node::new(py_none(), None);

        tree.lock().unwrap().add_child(child_node.clone(), None);
        tree.lock().unwrap().add_child(childs_child_node.clone(), Some(child_node.clone()));

        let ancestors = tree.lock().unwrap().get_ancestors(&childs_child_node);

        assert!(ancestors.iter().any(|ancestor| Arc::ptr_eq(ancestor, &child_node)));
        assert!(ancestors.iter().any(|ancestor| Arc::ptr_eq(ancestor, &tree.lock().unwrap().root)));
    }

    #[test]
    fn test_move_node_from_two_deep_to_one_deep_mt(){
        let tree = Tree::new(None);
        let child_node = Node::new(py_none(), None);
        let childs_child_node = Node::new(py_none(), None);

        {
            let tree_guard = tree.lock().unwrap();
            tree_guard.add_child(child_node.clone(), None);
            tree_guard.add_child(childs_child_node.clone(), Some(child_node.clone()));

            let root_clone = tree_guard.root.clone();
            tree_guard.move_node(&childs_child_node, &root_clone);
        }

        let children = tree.lock().unwrap().root.lock().unwrap().children.lock().unwrap().clone();

        assert!(children.iter().any(|child| Arc::ptr_eq(&child, &child_node)));
        assert!(children.iter().any(|child| Arc::ptr_eq(&child, &childs_child_node)));
    }
}