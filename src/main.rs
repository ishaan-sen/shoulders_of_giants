#![warn(clippy::pedantic)]

mod adj_dag;
mod dag;
mod edge_list_dag;
mod linked_dag;

use std::borrow::Cow;
use std::collections::HashSet;
use std::io::{self, Write};
use std::rc::Rc;

use adj_dag::AdjDag;
use dag::{earliest_common_descendant, last_common_ancestor, Dag};
use edge_list_dag::EdgeListDag;
use linked_dag::LinkedDag;

#[derive(Clone)]
pub struct CSVRecord {
    pub id: Rc<str>,
    pub title: Box<str>,
    pub abstract_text: Box<str>,
    pub references: HashSet<Rc<str>>,
}

#[derive(Clone)]
pub struct Paper {
    pub id: Rc<str>,
    pub title: Box<str>,
    pub abstract_text: Box<str>,
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

fn list_related_papers(dag: &impl Dag<NodeWeight = Paper>) {
    print!("Enter paper ID: ");
    io::stdout().flush().ok(); // doesn't print consistently without this
    let mut id = String::new();
    if io::stdin().read_line(&mut id).is_err() {
        return;
    }
    let id = id.trim();

    let node_id = dag.find_node(|_, w| *w.id == *id).expect("Paper not found");

    let incoming: Vec<&Paper> = dag.neighbors_back(&node_id).map(|nid| &dag[&nid]).collect();
    let outgoing: Vec<&Paper> = dag.neighbors(&node_id).map(|nid| &dag[&nid]).collect();

    print_paper_table(&format!("Incoming (citations to {id})"), &incoming);
    println!();
    print_paper_table(&format!("Outgoing (references from {id})"), &outgoing);
}

fn latest_common_ancestor_op(dag: &impl Dag<NodeWeight = Paper>) {
    print!("Enter first paper ID: ");
    io::stdout().flush().ok();
    let mut a = String::new();
    if io::stdin().read_line(&mut a).is_err() {
        return;
    }
    let a = a.trim();

    print!("Enter second paper ID: ");
    io::stdout().flush().ok();
    let mut b = String::new();
    if io::stdin().read_line(&mut b).is_err() {
        return;
    }
    let b = b.trim();

    let node_a = dag
        .find_node(|_, w| *w.id == *a)
        .expect("First paper not found");
    let node_b = dag
        .find_node(|_, w| *w.id == *b)
        .expect("Second paper not found");

    let ancestors: Vec<&Paper> = last_common_ancestor(dag, &node_a, &node_b).collect();

    if ancestors.is_empty() {
        println!("No common ancestor found.");
    } else {
        println!("\nLatest common ancestor(s):");
        for p in &ancestors {
            println!(
                "  ID: {} | Title: {} | Abstract: {}",
                p.id,
                if p.title.is_empty() {
                    "(none)"
                } else {
                    &p.title
                },
                if p.abstract_text.is_empty() {
                    "(none)"
                } else {
                    &p.abstract_text
                }
            );
        }
    }
}

fn earliest_common_descendant_op(dag: &impl Dag<NodeWeight = Paper>) {
    print!("Enter first paper ID: ");
    io::stdout().flush().ok();
    let mut a = String::new();
    if io::stdin().read_line(&mut a).is_err() {
        return;
    }
    let a = a.trim();

    print!("Enter second paper ID: ");
    io::stdout().flush().ok();
    let mut b = String::new();
    if io::stdin().read_line(&mut b).is_err() {
        return;
    }
    let b = b.trim();

    let node_a = dag
        .find_node(|_, w| *w.id == *a)
        .expect("First paper not found");
    let node_b = dag
        .find_node(|_, w| *w.id == *b)
        .expect("Second paper not found");

    let ancestors: Vec<&Paper> = earliest_common_descendant(dag, &node_a, &node_b).collect();

    if ancestors.is_empty() {
        println!("No common ancestor found.");
    } else {
        println!("\nEarliest common descendant(s):");
        for p in &ancestors {
            println!(
                "  ID: {} | Title: {} | Abstract: {}",
                p.id,
                if p.title.is_empty() {
                    "(none)"
                } else {
                    &p.title
                },
                if p.abstract_text.is_empty() {
                    "(none)"
                } else {
                    &p.abstract_text
                }
            );
        }
    }
}

enum ActiveDag<'a> {
    Adj(&'a AdjDag<Paper>),
    Linked(&'a LinkedDag<Paper>),
    EdgeList(&'a EdgeListDag<Paper>),
}

impl<'a> ActiveDag<'a> {
    fn name(&self) -> &'static str {
        match self {
            ActiveDag::Adj(_) => "adjacency matrix",
            ActiveDag::Linked(_) => "pointer-based",
            ActiveDag::EdgeList(_) => "edge list",
        }
    }

    fn list_related(&self) {
        match self {
            ActiveDag::Adj(d) => list_related_papers(*d),
            ActiveDag::Linked(d) => list_related_papers(*d),
            ActiveDag::EdgeList(d) => list_related_papers(*d),
        }
    }

    fn lca(&self) {
        match self {
            ActiveDag::Adj(d) => latest_common_ancestor_op(*d),
            ActiveDag::Linked(d) => latest_common_ancestor_op(*d),
            ActiveDag::EdgeList(d) => latest_common_ancestor_op(*d),
        }
    }

    fn ecd(&self) {
        match self {
            ActiveDag::Adj(d) => earliest_common_descendant_op(*d),
            ActiveDag::Linked(d) => earliest_common_descendant_op(*d),
            ActiveDag::EdgeList(d) => earliest_common_descendant_op(*d),
        }
    }

    fn search(&self) {
        match self {
            ActiveDag::Adj(d) => search_papers(*d),
            ActiveDag::Linked(d) => search_papers(*d),
            ActiveDag::EdgeList(d) => search_papers(*d),
        }
    }
}

fn filter_disconnected(records: Vec<CSVRecord>) -> Vec<CSVRecord> {
    let ids: HashSet<Rc<str>> = records.iter().map(|r| r.id.clone()).collect();

    let mut has_incoming: HashSet<Rc<str>> = HashSet::new();
    for r in &records {
        for refr in &r.references {
            if ids.contains(refr) {
                has_incoming.insert(refr.clone());
            }
        }
    }

    records
        .into_iter()
        .filter(|r| {
            let has_outgoing = r.references.iter().any(|refr| ids.contains(refr));
            has_outgoing || has_incoming.contains(&r.id)
        })
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let num_lines: usize = args[1].parse().unwrap();
    println!("loading csv");

    let all_records = load_csv("dataset/dblp-v10.csv");

    let records: Vec<CSVRecord> =
        filter_disconnected(all_records.into_iter().take(num_lines).collect());

    let n_records = records.len();

    println!("spared {n_records} papers");
    println!("building linkeddag");
    let linked: LinkedDag<Paper> = records.iter().cloned().collect();

    println!("building adjdag");
    let adj: AdjDag<Paper> = records.iter().cloned().collect();

    println!("building edgelistdag");
    let edge_list: EdgeListDag<Paper> = records.into_iter().collect();

    let mut active = ActiveDag::Adj(&adj);

    menu_loop(&mut active, &adj, &linked, &edge_list);
}

fn menu_loop<'a>(
    active: &mut ActiveDag<'a>,
    adj: &'a AdjDag<Paper>,
    linked: &'a LinkedDag<Paper>,
    edge_list: &'a EdgeListDag<Paper>,
) {
    loop {
        println!();
        println!("Select an operation:");
        println!("  1. List related papers");
        println!("  2. Latest common ancestor");
        println!("  3. Earliest common descendant");
        println!("  4. Search papers");
        println!("  5. Select DAG (currently {})", active.name());
        println!("  6. Exit");
        print!("> ");
        io::stdout().flush().ok();

        let mut choice = String::new();
        if io::stdin().read_line(&mut choice).is_err() {
            break;
        }

        match choice.trim() {
            "1" => active.list_related(),
            "2" => active.lca(),
            "3" => active.ecd(),
            "4" => active.search(),
            "5" => {
                *active = match active {
                    ActiveDag::Adj(_) => ActiveDag::Linked(linked),
                    ActiveDag::Linked(_) => ActiveDag::EdgeList(edge_list),
                    ActiveDag::EdgeList(_) => ActiveDag::Adj(adj),
                };
                println!("Switched to {}.", active.name());
            }
            "6" => {
                println!("Exiting.");
                break;
            }
            other => println!("Unknown option \"{other}\""),
        }
    }
}

// Since this function might or might not need to modify its input, it returns
// a clone-on-write (`Cow`) objects, which could wrap either a borrowed `&str`
// or an owned `String`
fn truncate_str(s: &str, max_chars: usize) -> Cow<'_, str> {
    if s.chars().count() > max_chars {
        let t: String = s.chars().take(max_chars).collect();
        format!("{t}...").into()
    } else {
        s.into()
    }
}

fn print_paper_table(title: &str, papers: &[&Paper]) {
    println!("{title}");
    println!("{:-<1$}", "", 80);
    println!("{:<30} {:<20} {:<10}", "Title", "ID", "Abstract"); // I really like this syntax actually, so much better than cpp
    println!("{:-<1$}", "", 80);
    for p in papers {
        println!(
            "{:<30} {:<20} {:<10}",
            truncate_str(&p.title, 27),
            p.id,
            truncate_str(&p.abstract_text, 40)
        );
    }
}

fn search_papers(dag: &impl Dag<NodeWeight = Paper>) {
    print!("Enter title substring to search: ");
    io::stdout().flush().ok();
    let mut query = String::new();
    if io::stdin().read_line(&mut query).is_err() {
        return;
    }
    let query = query.trim();

    let ids: Vec<&Paper> = dag
        .find_nodes(|_, w| w.title.contains(query))
        .map(|nid| &dag[&nid])
        .collect();

    if ids.is_empty() {
        println!("No papers found");
    } else {
        print_paper_table("Found paper(s)", &ids);
    }
}