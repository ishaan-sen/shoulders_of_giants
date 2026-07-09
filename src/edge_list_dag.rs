use super::Node;
use std::collections::HashMap;
use std::rc::Rc;

pub struct EdgeListDag {
    pub nodes: Vec<Node>,
    pub edges: Vec<(usize, usize)>,
    pub node_map: HashMap<Rc<str>, usize>,
}

impl EdgeListDag {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> usize {
        if let Some(&idx) = self.node_map.get(&node.id) {
            if self.nodes[idx].is_dummy && !node.is_dummy {
                self.nodes[idx] = node;
            }
            idx
        } else {
            let idx = self.nodes.len();
            // Cloning an Rc just increments the reference counter, it is very fast!
            self.node_map.insert(node.id.clone(), idx); 
            self.nodes.push(node);
            idx
        }
    }

    pub fn add_edge(&mut self, citer_idx: usize, cited_idx: usize) {
        self.edges.push((citer_idx, cited_idx));
    }

    pub fn get_connected_edges(&self, target_id: &str) -> Option<(Vec<&Node>, Vec<&Node>)> {
        let &target_idx = self.node_map.get(target_id)?;
        
        let mut incoming = Vec::new();
        let mut outgoing = Vec::new();

        for &(citer, cited) in &self.edges {
            if citer == target_idx {
                outgoing.push(&self.nodes[cited]);
            } else if cited == target_idx {
                incoming.push(&self.nodes[citer]);
            }
        }

        Some((incoming, outgoing))
    }
}