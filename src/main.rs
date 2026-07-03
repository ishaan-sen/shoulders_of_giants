#![warn(clippy::pedantic)]

mod linked_dag;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;

struct Node {
    id: String,
    title: String,
    abstract_text: String,
    is_dummy: bool,
}

struct CSVRecord {
    id: String,
    title: String,
    abstract_text: String,
    references: Vec<String>,
}

fn parse_references(refs: &str) -> Vec<String> {
    refs.trim_matches(|c| c == '[' || c == ']')
        .split(", ")
        .map(|s| s.trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn load_csv(path: &str) -> Vec<CSVRecord> {
    let mut reader = csv::Reader::from_path(path).unwrap();
    reader
        .records()
        .filter_map(|rec| rec.ok())
        .map(|rec| CSVRecord {
            abstract_text: rec[0].to_string(),
            title: rec[4].to_string(),
            references: parse_references(&rec[3]),
            id: rec[7].to_string(),
        })
        .collect()
}

fn build_graph(records: &[CSVRecord]) -> DiGraph<Node, ()> {
    let mut graph = DiGraph::<Node, ()>::new();
    let mut node_map: HashMap<String, NodeIndex> = HashMap::new();

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
                    title: String::new(),
                    abstract_text: String::new(),
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
