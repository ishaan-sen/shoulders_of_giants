use crate::dag::Dag;
use crate::Paper;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

pub struct EdgeListDag<T> {
    pub nodes: Vec<T>,
    pub edges: Vec<(usize, usize)>,
}

impl<T> EdgeListDag<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, citer_idx: usize, cited_idx: usize) {
        self.edges.push((citer_idx, cited_idx));
    }
}

impl FromIterator<crate::CSVRecord> for EdgeListDag<Paper> {
    fn from_iter<T: IntoIterator<Item = crate::CSVRecord>>(iter: T) -> Self {
        let mut dag = EdgeListDag::new();
        let records: Vec<_> = iter.into_iter().collect();
        
        let mut node_map = HashMap::new();

        for rec in &records {
            let paper = Paper {
                id: rec.id.clone(),
                title: rec.title.clone(),
                abstract_text: rec.abstract_text.clone(),
            };
            let idx = dag.nodes.len();
            node_map.insert(paper.id.clone(), idx);
            dag.nodes.push(paper);
        }

        for rec in &records {
            if let Some(&citer_idx) = node_map.get(&rec.id) {
                for refr in &rec.references {
                    if let Some(&cited_idx) = node_map.get(refr) {
                        dag.add_edge(citer_idx, cited_idx);
                    }
                }
            }
        }

        dag
    }
}

impl<T> Index<&usize> for EdgeListDag<T> {
    type Output = T;

    fn index(&self, index: &usize) -> &Self::Output {
        &self.nodes[*index]
    }
}

impl<T> IndexMut<&usize> for EdgeListDag<T> {
    fn index_mut(&mut self, index: &usize) -> &mut Self::Output {
        &mut self.nodes[*index]
    }
}

impl<T> Dag for EdgeListDag<T> {
    type NodeWeight = T;
    type NodeId = usize;

    fn neighbors(&self, node: &Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        let target = *node;
        self.edges.iter().filter_map(move |&(citer, cited)| {
            if citer == target {
                Some(cited)
            } else {
                None
            }
        })
    }

    fn neighbors_back(&self, node: &Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        let target = *node;
        self.edges.iter().filter_map(move |&(citer, cited)| {
            if cited == target {
                Some(citer)
            } else {
                None
            }
        })
    }

    fn find_nodes(
        &self,
        mut func: impl FnMut(&Self::NodeId, &Self::NodeWeight) -> bool,
    ) -> impl Iterator<Item = Self::NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(move |(id, node)| if func(&id, node) { Some(id) } else { None })
    }

    fn find_node(
        &self,
        mut func: impl FnMut(&Self::NodeId, &Self::NodeWeight) -> bool,
    ) -> Option<Self::NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .find_map(|(id, node)| if func(&id, node) { Some(id) } else { None })
    }

    fn get(&self, id: &Self::NodeId) -> Option<&Self::NodeWeight> {
        self.nodes.get(*id)
    }

    fn get_mut(&mut self, id: &Self::NodeId) -> Option<&mut Self::NodeWeight> {
        self.nodes.get_mut(*id)
    }
}

impl<T> Default for EdgeListDag<T> {
    fn default() -> Self {
        Self::new()
    }
}