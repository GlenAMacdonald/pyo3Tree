use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pprof::criterion::{Output, PProfProfiler};

use std::sync::{Arc, Mutex};
use rand::thread_rng;
use rand::seq::SliceRandom;
use tree_rs::{Tree, Node};

fn add_node_to_empty_tree(){
    let tree = Tree::new(None);
    let child_node = Node::new(None);

    tree.lock().unwrap().add_child(child_node.clone(), None);
    
    let child_parent_id = child_node.lock().unwrap().parent.as_ref().unwrap().upgrade().unwrap().lock().unwrap().id.clone();
    
    let root_id = tree.lock().unwrap().root.lock().unwrap().id.clone();
}

fn add_many_children(){
    let n_children = 5000;
    let mut members: Vec<Arc<Mutex<Node>>> = Vec::with_capacity(n_children);
    let mut rng = thread_rng();

    let tree = Tree::new(None);

    for i in 2..n_children {
        let child_node = Node::new(None);
        tree.lock().unwrap().add_child(child_node.clone(), members.choose(&mut rng).map(|n| n.clone()));
        members.push(child_node.clone());
    }
}

fn add_node_two_deep(){
    let tree = Tree::new(None);
    let child_node = Node::new(None);
    let childs_child_node = Node::new(None);

    tree.lock().unwrap().add_child(child_node.clone(), None);
    tree.lock().unwrap().add_child(childs_child_node.clone(), Some(child_node.clone()));
}

fn find_by_id(){
    let tree = Tree::new(None);
    let child_node = Node::new(None);
    let childs_child_node = Node::new(None);

    {
        tree.lock().unwrap().add_child(child_node.clone(), None);
        tree.lock().unwrap().add_child(childs_child_node.clone(), Some(child_node.clone()));
    }
}

fn get_ancestors_on_two_deep_tree(){
    let tree = Tree::new(None);
    let child_node = Node::new(None);
    let childs_child_node = Node::new(None);

    tree.lock().unwrap().add_child(child_node.clone(), None);
    tree.lock().unwrap().add_child(childs_child_node.clone(), Some(child_node.clone()));

    let ancestors = tree.lock().unwrap().get_ancestors(&childs_child_node);
}

fn move_node_from_two_deep_to_one_deep(){
    let tree = Tree::new(None);
    let child_node = Node::new(None);
    let childs_child_node = Node::new(None);

    {
        let tree_guard = tree.lock().unwrap();
        tree_guard.add_child(child_node.clone(), None);
        tree_guard.add_child(childs_child_node.clone(), Some(child_node.clone()));

        let root_clone = tree_guard.root.clone();
        tree_guard.move_node(&childs_child_node, &root_clone);
    }
}

// add_node_to_empty_tree_mt
// add_node_two_deep_mt, find_by_id_mt, get_ancestors_on_two_deep_tree_mt
// move_node_from_two_deep_to_one_deep_m
fn single_functionality_benches(c: &mut Criterion) {
    c.bench_function("add node to empty tree", |b| b.iter(|| add_node_to_empty_tree()));
    c.bench_function("add node two deep", |b| b.iter(|| add_node_two_deep()));
    c.bench_function("find by id", |b| b.iter(|| find_by_id()));
    c.bench_function("get ancestors on two deep tree", |b| b.iter(|| get_ancestors_on_two_deep_tree()));
    c.bench_function("move node from two deep to one deep", |b| b.iter(|| move_node_from_two_deep_to_one_deep()));
}

fn bulk_runs_benches(c: &mut Criterion) {
    c.bench_function("Create many children", |b| b.iter(|| add_many_children()));
}

criterion_group!{
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = single_functionality_benches, bulk_runs_benches
}
criterion_main!(benches);