#![warn(clippy::pedantic)]

mod adj_dag;
mod linked_dag;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

struct Node {
    id: Rc<str>,
    title: Box<str>,
    abstract_text: Box<str>,
    is_dummy: bool,
}

struct CSVRecord {
    id: Rc<str>,
    title: Box<str>,
    abstract_text: Box<str>,
    references: HashSet<Rc<str>>,
}

fn parse_references(refs: &str) -> HashSet<Rc<str>> {
    refs.trim_matches(|c| c == '[' || c == ']')
        .split(", ")
        .map(|s| s.trim_matches('\''))
        .filter(|s| !s.is_empty())
        .map(Into::into)
        .collect()
}

fn load_csv(path: &str) -> Vec<CSVRecord> {
    let mut reader = csv::Reader::from_path(path).unwrap();
    reader
        .records()
        .filter_map(|rec| rec.ok())
        .map(|rec| CSVRecord {
            abstract_text: rec[0].into(),
            title: rec[4].into(),
            references: parse_references(&rec[3]),
            id: rec[7].into(),
        })
        .collect()
}

fn build_graph(records: &[CSVRecord]) -> DiGraph<Node, ()> {
    let mut graph = DiGraph::<Node, ()>::new();
    let mut node_map: HashMap<Rc<str>, NodeIndex> = HashMap::new();

    for r in records {
        let node_idx = graph.add_node(Node {
            id: r.id.clone(),
            title: r.title.clone(),
            abstract_text: r.abstract_text.clone(),
            is_dummy: false,
        });
        node_map.insert(r.id.clone(), node_idx);
    }

    for record in records {
        let citer_idx = *node_map.get(&record.id).unwrap();

        for reference in &record.references {
            let cited_idx = *node_map.entry(reference.clone()).or_insert_with(|| {
                graph.add_node(Node {
                    id: reference.clone(),
                    title: "".into(),
                    abstract_text: "".into(),
                    is_dummy: true,
                })
            });

            graph.add_edge(citer_idx, cited_idx, ());
        }
    }

    graph
}

fn main() {
    let records = load_csv("dataset/dblp-v10.csv");

    let graph = build_graph(&records);

    let total_nodes = graph.node_count();
    let real_count = graph.node_weights().filter(|n| !n.is_dummy).count();
    let dummy_count = total_nodes - real_count;
    let total_edges = graph.edge_count();

    println!(
        "total nodes: {} \nreal papers: {} \ndummy nodes: {} \ntotal edges: {}",
        total_nodes, real_count, dummy_count, total_edges
    );
}
