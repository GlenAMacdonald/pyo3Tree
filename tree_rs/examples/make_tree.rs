// https://github.com/tikv/pprof-rs/tree/master/examples - reference
// at the moment this creates a zero byte sized .svg file...

use tree_rs::{Tree, Node};
use std::{fs::File, sync::{Arc, Mutex}};
use rand::thread_rng;
use rand::seq::SliceRandom;

fn main(){
    let n_children = 5000;
    let mut members: Vec<Arc<Mutex<Node>>> = Vec::with_capacity(n_children);
    let mut rng = thread_rng();

    let guard = pprof::ProfilerGuard::new(1000).unwrap();

    let tree = Tree::new(None);

    for _i in 2..n_children {
        let child_node = Node::new(None);
        tree.lock().unwrap().add_child(child_node.clone(), members.choose(&mut rng).map(|n| n.clone()));
        members.push(child_node.clone());
    }

    if let Ok(report) = guard.report().build() {
        println!("report: {:?}", &report);
        let file = File::create("flamegraph.svg").unwrap();
        report.flamegraph(file).unwrap();
    };
}