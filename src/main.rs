#![warn(clippy::pedantic)]

mod adj_dag;
mod dag;
mod edge_list_dag;
mod linked_dag;

use edge_list_dag::EdgeListDag;
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

#[derive(Clone)]
struct Paper {
    id: Rc<str>,
    title: Box<str>,
    abstract_text: Box<str>,
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
        .filter_map(Result::ok)
        .map(|rec| CSVRecord {
            abstract_text: rec[0].into(),
            title: rec[4].into(),
            references: parse_references(&rec[3]),
            id: rec[7].into(),
        })
        .collect()
}

fn filter_csv(iter: impl IntoIterator<Item = CSVRecord>) -> Vec<CSVRecord> {
    let recs: Vec<CSVRecord> = iter.into_iter().collect();
    let mut point_count: HashMap<Rc<str>, (usize, usize)> = HashMap::new();
    for rec in &recs {
        for refr in &rec.references {
            point_count.entry(refr.clone()).or_default().0 += 1;
        }
        point_count.entry(rec.id.clone()).or_default().1 = rec.references.len();
    }
    recs.into_iter()
        .filter(|rec| point_count[&rec.id] != (0, 0))
        .collect()
}

fn build_graph(records: &[CSVRecord]) -> (DiGraph<Node, ()>, EdgeListDag) {
    let mut graph = DiGraph::<Node, ()>::new();
    let mut node_map: HashMap<Rc<str>, NodeIndex> = HashMap::new();

    let mut edge_dag = EdgeListDag::new();

    for r in records {
        let node_idx = graph.add_node(Node {
            id: r.id.clone(),
            title: r.title.clone(),
            abstract_text: r.abstract_text.clone(),
            is_dummy: false,
        });
        node_map.insert(r.id.clone(), node_idx);

        edge_dag.add_node(Node {
            id: r.id.clone(),
            title: r.title.clone(),
            abstract_text: r.abstract_text.clone(),
            is_dummy: false,
        });
    }

    for record in records {
        let citer_idx = *node_map.get(&record.id).unwrap();
        let edge_citer_idx = *edge_dag.node_map.get(&record.id).unwrap();

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

            let edge_cited_idx = match edge_dag.node_map.get(reference) {
                Some(&idx) => idx,
                None => edge_dag.add_node(Node {
                    id: reference.clone(),
                    title: "".into(),
                    abstract_text: "".into(),
                    is_dummy: true,
                }),
            };
            edge_dag.add_edge(edge_citer_idx, edge_cited_idx);
        }
    }

    (graph, edge_dag)
}

fn main() {
    let records = filter_csv(load_csv("dataset/dblp-v10.csv"));

    let (graph, edge_dag) = build_graph(&records);

    let total_nodes = graph.node_count();
    let real_count = graph.node_weights().filter(|n| !n.is_dummy).count();
    let dummy_count = total_nodes - real_count;
    let total_edges = graph.edge_count();

    println!(
        "total nodes: {total_nodes} \nreal papers: {real_count} \ndummy nodes: {dummy_count} \ntotal edges: {total_edges}"
    );

    println!(
        "\nEdge List DAG stats:\ntotal nodes: {}\ntotal edges: {}",
        edge_dag.nodes.len(),
        edge_dag.edges.len()
    );
}
