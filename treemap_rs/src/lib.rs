use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use anyhow::{Result, anyhow};

pub struct TreeMap {
    pub nodes: Arc<RwLock<HashMap<String,Arc<RwLock<NodeMap>>>>>,
    // TODO: Store root node id here, remove the clonky double handling below
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_add_node_to_empty_tree(){
        let tree = TreeMap::new(None);
        let child_node = NodeMap::new( None);

        tree.add_child(&child_node, None).unwrap();

        let child_parent_id = child_node.read().unwrap().parent.as_ref().unwrap().clone();

        let root_node = tree.nodes.read().unwrap().get("root").unwrap().clone();

        let root_guard = root_node.read().unwrap();

        let root_id = root_guard.id.clone();

        assert!(&child_parent_id == &root_id);

        let child_node_id = child_node.read().unwrap().id.clone();

        assert!(root_guard.children.iter().any(|child| child == &child_node_id));
    }

    #[test]
    fn test_add_node_two_deep(){
        let tree = TreeMap::new(None);
        let child_node = NodeMap::new(None);
        let childs_child_node = NodeMap::new( None);

        tree.add_child(&child_node, None).unwrap();
        tree.add_child(&childs_child_node, Some(&child_node)).unwrap();

        let childs_child_parent_id = childs_child_node.read().unwrap().parent.as_ref().unwrap().clone();

        let child_id = child_node.read().unwrap().id.clone();

        assert_eq!(childs_child_parent_id, child_id);

        assert!(child_node.read().unwrap().children.iter().any(|child| child == &childs_child_node.read().unwrap().id.as_ref()));
    }

    #[test]
    fn test_find_by_id(){
        let tree = TreeMap::new(None);
        let child_node = NodeMap::new( None);
        let childs_child_node = NodeMap::new( None);

        tree.add_child(&child_node, None).unwrap();
        tree.add_child(&childs_child_node, Some(&child_node)).unwrap();

        let childs_child_id =  {childs_child_node.read().unwrap().id.clone()};
        let found_child = {tree.find_by_id(&childs_child_id).unwrap().clone()};
        assert!(Arc::ptr_eq(&found_child,&childs_child_node));
    }

    #[test]
    fn test_get_ancestors_on_two_deep_tree(){
        let tree = TreeMap::new(None);
        let child_node = NodeMap::new( None);
        let childs_child_node = NodeMap::new( None);

        tree.add_child(&child_node, None).unwrap();
        tree.add_child(&childs_child_node, Some(&child_node)).unwrap();

        let grand_child_ancestors = tree.get_ancestors(&childs_child_node).unwrap();
        let root_node = tree.nodes.read().unwrap().get("root").unwrap().clone();

        assert!(grand_child_ancestors.iter().any(|ancestor| Arc::ptr_eq(ancestor, &child_node)));
        assert!(grand_child_ancestors.iter().any(|ancestor| Arc::ptr_eq(ancestor, &root_node)));
    }

    #[test]
    fn test_move_node_from_two_deep_to_one_deep(){
        let tree = TreeMap::new(None);
        let child_node = NodeMap::new( None);
        let childs_child_node = NodeMap::new( None);

        tree.add_child(&child_node, None).unwrap();
        tree.add_child(&childs_child_node, Some(&child_node)).unwrap();

        let root_clone = tree.nodes.read().unwrap().get("root").unwrap().clone();
        tree.move_node(&childs_child_node, &root_clone).unwrap();

        let children = root_clone.read().unwrap().children.clone();

        assert!(children.iter().any(|child| child == &child_node.read().unwrap().id));
        assert!(children.iter().any(|child| child == &childs_child_node.read().unwrap().id));
    }
}