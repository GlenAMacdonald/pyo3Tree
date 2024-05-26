use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use pyo3::{prelude::*, PyObject, Python, ToPyObject};
use pyo3::types::{PyDict, PyList};
use tree_rs::{Node as Node_rs, Tree as Tree_rs, NodeMap as NodeMap_rs, TreeMap as TreeMap_rs};

use dashmap::DashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref DATA_MAP: DashMap<String, PyObject> = DashMap::new();
    // TODO create a node cache for node_wrapper generation and pass Python only a weak reference, take ownership from this cache when added to the tree.
    static ref TREE_MAP: Arc<RwLock<TreeMap_rs>> = Arc::new(RwLock::new(TreeMap_rs::new(None)));
}

#[pyclass]
#[pyo3(name = "TreeMap")]
struct TreeMapWrapper(Arc<RwLock<TreeMap_rs>>);

#[pymethods]
impl TreeMapWrapper {
    #[new]
    fn new(root: Option<NodeMapWrapper>) -> Self {
        match root {
            Some(wrapped_node) => {
                let node = wrapped_node.0;
                let new_tree = TreeMap_rs::new(Some(node));
                TREE_MAP.write().unwrap().nodes = new_tree.nodes;
                TreeMapWrapper(TREE_MAP.clone())
            },
            None => TreeMapWrapper(TREE_MAP.clone())
        }
    }

    #[getter]
    fn get_root(&self) -> PyResult<NodeMapWrapper> {
        Ok(NodeMapWrapper(self.0.read().unwrap().nodes.read().unwrap().get("root").unwrap().clone()))
    }

    fn add(&self, child: NodeMapWrapper, parent_node: Option<NodeMapWrapper>) -> PyResult<()>{
        let result = match parent_node {
            Some(parent) => {self.0.write().unwrap().add_child(&child.0, Some(&parent.0))},
            None => {self.0.write().unwrap().add_child(&child.0, None)}
        };

        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to add child: {}",e)))
        }
    }

    pub fn find_by_id(&self, id: String) -> PyResult<NodeMapWrapper> {
        Ok(NodeMapWrapper(self.0.read().unwrap().find_by_id(&id).unwrap()))
    }

    pub fn move_node(&self, tgt_node: NodeMapWrapper, new_parent_node: NodeMapWrapper) -> PyResult<()> {
        match self.0.write().unwrap().move_node(&tgt_node.0, &new_parent_node.0) {
            Ok(()) => return Ok(()),
            Err(e) => return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to move node: {}",e)))
        };
    }

    pub fn get_ancestors(&self, node: NodeMapWrapper) -> PyResult<Vec<NodeMapWrapper>> {
        if let Ok(ancestors) = self.0.read().unwrap().get_ancestors(&node.0){
            let ancestors_nodemap: Vec<NodeMapWrapper> = ancestors
                .into_iter()
                .map(NodeMapWrapper)
                .collect();

            return Ok(ancestors_nodemap);
        }
        return Err(pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to get ancestors for node with id: {}", node.0.read().unwrap().id.clone())))
    }

    #[staticmethod]
    pub fn load(py: Python, python_tree: &Bound<PyDict>) -> PyResult<Self> {
        // Clear data
        DATA_MAP.clear();
        // Define queue as containing a node (defined as a PyDict) and its parent_id
        let mut queue: VecDeque<(Bound<PyDict>,Arc<RwLock<NodeMap_rs>>)> = VecDeque::new();
        // initial parent is 'root'
        let id = extract_id_from_pyobject(python_tree);
        import_node_data_from_pyobject(py, &id, python_tree);
        let root: Arc<RwLock<NodeMap_rs>> = Arc::new(RwLock::new(NodeMap_rs {id, children: Vec::with_capacity(5), parent: None}));
        
        // Create the tree with this root node
        let tree = TreeMap_rs::new(Some(root.clone()));
        let children_dicts = extract_children_from_pyobject(python_tree.clone());
            queue.extend(children_dicts.iter().map(|child| (child.clone(), root.clone())));

        while let Some((obj,parent)) = queue.pop_front() {
            let id = extract_id_from_pyobject(&obj);
            import_node_data_from_pyobject(py, &id, &obj);
            let node: Arc<RwLock<NodeMap_rs>> = Arc::new(RwLock::new(NodeMap_rs {id, children: Vec::with_capacity(5), parent: None}));
            tree.add_child(&node, Some(&parent)).unwrap();

            let children_dicts = extract_children_from_pyobject(obj);
            queue.extend(children_dicts.iter().map(|child| (child.clone(), node.clone())));

        };

        TREE_MAP.write().unwrap().nodes = tree.nodes;
        Ok(TreeMapWrapper(TREE_MAP.clone()))
    }

    pub fn export(&self, py: Python) -> PyResult<PyObject> {
        Ok(set_py_dict_recursively_map(py, self.0.read().unwrap().nodes.read().unwrap().get("root").unwrap()))
    }
}

fn import_node_data_from_pyobject(py: Python, id: &String, obj: &Bound<PyDict>) {
    match obj.get_item("data") {
        Ok(Some(value)) => (DATA_MAP.insert(id.clone(),value.to_object(py))),
        Ok(None) => None,
        Err(err) => None,
    };
}

fn extract_id_from_pyobject(obj: &Bound<PyDict>) -> String {
    let id = match obj.get_item("id") {
        Ok(Some(value)) => value.extract::<String>().map_err(|_| pyo3::exceptions::PyTypeError::new_err("Failed to extract 'id'")),
        Ok(None) => Err(pyo3::exceptions::PyTypeError::new_err("'id' key not found")),
        Err(err) => Err(err),
    };

    return id.unwrap()
}

fn extract_children_from_pyobject(obj: Bound<PyDict>) -> Vec<Bound<PyDict>> {
    let children = match obj.get_item("children") {
        Ok(Some(value)) => value.extract::<Vec<Bound<PyDict>>>(),
        Ok(None) => Ok(Vec::new()),
        Err(err) => Err(err),
    };

    return children.unwrap()
}

fn set_py_dict_recursively_map(py: Python, node: &Arc<RwLock<NodeMap_rs>>) -> PyObject {
    let node_lock = node.read().unwrap();
    let py_dict = PyDict::new_bound(py);

    py_dict.set_item("id", node_lock.id.clone()).unwrap();

    if let Some(data) = DATA_MAP.get(&node_lock.id) {
        py_dict.set_item("data", data.clone()).unwrap();
    }

    if node_lock.children.len() > 0 {
        let children_list = PyList::new_bound(py, node_lock.children.iter().map(|child| {
            set_py_dict_recursively_map(py, TREE_MAP.read().unwrap().nodes.read().unwrap().get(child).unwrap())
        }));
        py_dict.set_item("children", children_list).unwrap();
    }   

    py_dict.to_object(py)
}

#[pyclass]
#[pyo3(name = "Tree")]
#[derive(Clone)]
struct TreeWrapper(Arc<Mutex<Tree_rs>>);

#[pymethods]
impl TreeWrapper {
    #[new]
    fn new(root: Option<NodeWrapper>) -> Self {
        match root {
            Some(wrapped_node) => TreeWrapper(Tree_rs::new(Some(wrapped_node.0.clone()))),
            None => TreeWrapper(Tree_rs::new(None))
        }
    }

    #[getter]
    fn get_root(&self) -> PyResult<NodeWrapper> {
        let root = self.0.lock().unwrap().root.clone();
        Ok(NodeWrapper(root))
    }

    pub fn add(&self, child: NodeWrapper, parent: Option<NodeWrapper>) -> PyResult<()> {
        match parent {
            Some(parent_node) => {self.0.lock().unwrap().add_child(child.0.clone(), Some(parent_node.0.clone()))},
            None => {self.0.lock().unwrap().add_child(child.0.clone(), None)}
        }
        Ok(())
    }

    pub fn find_by_id(&self, id: String) -> PyResult<NodeWrapper> {
        Ok(NodeWrapper(self.0.lock().unwrap().find_by_id(&id).unwrap()))
    }

    pub fn move_node(&self, tgt_node: NodeWrapper, new_parent_node: NodeWrapper) -> PyResult<()> {
        self.0.lock().unwrap().move_node(&tgt_node.0, &new_parent_node.0);
        Ok(())
    }

    pub fn get_ancestors(&self, node: NodeWrapper) -> PyResult<Vec<NodeWrapper>> {
        let ancestors = &self.0.lock().unwrap().get_ancestors(&node.0);
        let mut wrapped_ancestors: Vec<NodeWrapper> = vec![];
        for ancestor in ancestors.iter(){
            wrapped_ancestors.push(NodeWrapper(ancestor.clone()));
        }
        Ok(wrapped_ancestors)
    }

    #[staticmethod]
    pub fn load(py: Python, python_tree: &Bound<PyDict>) -> PyResult<Self> {
        let big_node = load_py_tree(py, python_tree).unwrap();
        set_parents_recursively_from_py_tree(big_node.clone(), None);
        Ok(TreeWrapper(Tree_rs::new(Some(big_node))))
    }

    pub fn export(&self, py: Python) -> PyResult<PyObject> {
        Ok(set_py_dict_recursively(py, self.0.lock().unwrap().root.clone()))
    }
}   

fn set_parents_recursively_from_py_tree(node: Arc<Mutex<Node_rs>>, parent: Option<Arc<Mutex<Node_rs>>>) {
    let mut node_guard = node.lock().unwrap();
    if let Some(parent_arc) = parent {
        node_guard.parent.replace(Arc::downgrade(&parent_arc));
    }

    let children = node_guard.children.clone();
    for child in children.lock().unwrap().iter() {
        set_parents_recursively_from_py_tree(child.clone(), Some(node.clone()));
    }
}

fn load_py_tree(py:Python<'_>, obj: &Bound<PyDict>) -> PyResult<Arc<Mutex<Node_rs>>> {
    let id = match obj.get_item("id") {
        Ok(Some(value)) => value.extract::<String>().map_err(|_| pyo3::exceptions::PyTypeError::new_err("Failed to extract 'id'")),
        Ok(None) => Err(pyo3::exceptions::PyTypeError::new_err("'id' key not found")),
        Err(err) => Err(err),
    }?;

    let data = match obj.get_item("data") {
        Ok(Some(value)) => Ok(value.to_object(py)),
        Ok(None) => Ok(py.None()),
        Err(err) => Err(err),
    }?;

    // parent is not expected or needed in the incoming PyObject, it is inferred from the structure

    let children = match obj.get_item("children") {
        Ok(Some(value)) => value.extract::<Vec<Bound<PyDict>>>(),
        Ok(None) => Ok(Vec::new()),
        Err(err) => Err(err),
    }?;

    let node_children: Arc<Mutex<Vec<Arc<Mutex<Node_rs>>>>> = Arc::new(Mutex::new(vec![]));
    if children.len() > 0 {
        for item in children.iter() {
            node_children.lock().unwrap().push(load_py_tree(py, item).unwrap())
        }
    }

    Ok(Arc::new(Mutex::new(Node_rs{id, data, children: node_children, parent: None})))
}

fn set_py_dict_recursively(py: Python, node: Arc<Mutex<Node_rs>>) -> PyObject {
    let node_lock = node.lock().unwrap();
    let py_dict = PyDict::new_bound(py);

    py_dict.set_item("id", node_lock.id.clone()).unwrap();

    if !node_lock.data.is_none(py) {
        py_dict.set_item("data", node_lock.data.clone()).unwrap();
    }

    let children_lock = node_lock.children.lock().unwrap();
    if !children_lock.is_empty() {
        let children_list = PyList::new_bound(py, children_lock.iter().map(|child| {
            set_py_dict_recursively(py, child.clone())
        }));
        py_dict.set_item("children", children_list).unwrap();
    }   

    py_dict.to_object(py)
}

#[pyclass]
#[pyo3(name = "NodeMap")]
#[derive(Clone)]
struct NodeMapWrapper(Arc<RwLock<NodeMap_rs>>);

// impl Drop for NodeMapWrapper {
//     fn drop(&mut self){
//         DATA_MAP.remove(&self.0.read().unwrap().id);
//     }
// }

#[pymethods]
impl NodeMapWrapper {
    #[new]
    fn new(data: Option<PyObject>) -> Self {
        let node = NodeMap_rs::new(None);
        if let Some(value) = data {
            DATA_MAP.insert(node.read().unwrap().id.clone(), value);
        }
        NodeMapWrapper(node)
    }

    #[getter]
    fn get_id(&self) -> PyResult<String>{
        Ok(self.0.read().unwrap().id.clone())
    }

    #[getter]
    fn get_data(&self) -> PyResult<PyObject>{
        Ok(DATA_MAP.get(&self.0.read().unwrap().id).unwrap().clone())
    }

    #[setter]
    fn set_data(&self, data: Option<PyObject>) -> PyResult<()> {
        if let Some(value) = data {
            DATA_MAP.insert(self.0.read().unwrap().id.clone(), value);
        }
        Ok(())
    }

    #[getter]
    fn get_children(&self) -> PyResult<Vec<NodeMapWrapper>> {
        let mut children: Vec<NodeMapWrapper> = Vec::with_capacity(50);

        let tree_map_node = TREE_MAP.read().unwrap();
        let nodes_guard = tree_map_node.nodes.read().unwrap();

        let node_guard = self.0.read().unwrap();
        for child_id in &node_guard.children{
            children.push(NodeMapWrapper(nodes_guard.get(child_id).unwrap().clone()));
        };
        children.shrink_to_fit();
        Ok(children)
    }

    #[getter]
    fn get_parent(&self) -> PyResult<NodeMapWrapper> {
        let tree_map_node = TREE_MAP.read().unwrap();
        let nodes_guard = tree_map_node.nodes.read().unwrap();
        Ok(NodeMapWrapper(nodes_guard.get(self.0.read().unwrap().parent.as_ref().unwrap()).unwrap().clone()))
    }
}

#[pyclass]
#[pyo3(name = "Node")]
#[derive(Clone)]
struct NodeWrapper(Arc<Mutex<Node_rs>>);

#[pymethods]
impl NodeWrapper {
    #[new]
    fn new (data: Option<PyObject>) -> Self {
        match data {
            Some(value) => NodeWrapper(Node_rs::new(value, None)),
            None => NodeWrapper(Node_rs::new(py_none(),None))
        }
    }

    #[getter]
    fn get_id(&self) -> PyResult<String> {
        Ok(self.0.lock().unwrap().id.clone())
    }

    #[getter]
    fn get_data(&self, py: Python) -> PyResult<PyObject> {
        let node = self.0.lock().unwrap();
        Ok(DATA_MAP.get(&node.id).unwrap().clone())
    }

    #[setter]
    fn set_data(&self, data: PyObject) -> PyResult<()> {
        let node = self.0.lock().unwrap();
        DATA_MAP.insert(node.id.clone(), data);
        Ok(())
    }

    #[getter]
    fn get_children(&self) -> PyResult<Vec<NodeWrapper>> {
        let node = self.0.lock().unwrap();
        let mut node_children: Vec<NodeWrapper> = vec![];
        for child in node.children.lock().unwrap().iter() {
            node_children.push(NodeWrapper(child.clone()));
        };
        Ok(node_children)
    }

    #[getter]
    fn get_parent(&self) -> PyResult<Option<NodeWrapper>> {
        if let Some(weak_parent) = &self.0.lock().unwrap().parent {
            Ok(Some(NodeWrapper(weak_parent.upgrade().unwrap())))
        } else {
            Ok(None)
        }
    }
}

pub fn py_none() -> PyObject {
    Python::with_gil(|py| {
        py.None().to_object(py)
    })
}

#[pymodule]
fn pyo3Tree(_: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<NodeWrapper>()?;
    m.add_class::<TreeWrapper>()?;
    m.add_class::<NodeMapWrapper>()?;
    m.add_class::<TreeMapWrapper>()?;
    Ok(())
}