use crate::Paper;
use std::{collections::HashSet, ops::IndexMut, rc::Rc};

pub trait Dag: for<'id> IndexMut<&'id Self::NodeId, Output = Self::NodeWeight> {
    type NodeWeight;
    type NodeId;
    /// Return forward-neighbors of this node in no particular order
    fn neighbors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;
    /// Return reverse-neighbors of this node in no particular order
    fn neighbors_back(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;
    /// Return IDs of nodes matching some criterion
    fn find_nodes(
        &self,
        func: impl FnMut(&Self::NodeId, &Self::NodeWeight) -> bool,
    ) -> impl Iterator<Item = Self::NodeId>;
    /// Return the first ID matching some criterion
    fn find_node(
        &self,
        func: impl FnMut(&Self::NodeId, &Self::NodeWeight) -> bool,
    ) -> Option<Self::NodeId>;
    fn get(&self, id: &Self::NodeId) -> Option<&Self::NodeWeight>;
    fn get_mut(&mut self, id: &Self::NodeId) -> Option<&mut Self::NodeWeight>;
    // what other functions go here?
}

fn find_by_id(dag: &impl Dag<NodeWeight = Paper>, id: Rc<str>) -> Option<&Paper> {
    dag.find_node(|_, weight| weight.id == id)
        .map(|node_id| &dag[&node_id])
}

// fn earliest_common_descendant(dag: &impl Dag<NodeWeight = Paper>) -> impl Iterator<Item = Paper> {
//     todo!()
// }
fn last_common_ancestor<T: Eq + std::hash::Hash + Clone>(
    dag: &impl Dag<NodeWeight = Paper, NodeId = T>,
    a: T,
    b: T,
) -> impl Iterator<Item = &Paper> {
    let mut visited_a: HashSet<T> = HashSet::new();
    let mut visited_b: HashSet<T> = HashSet::new();
    let mut check_a: HashSet<T> = dag.neighbors(a).collect();
    let mut check_b: HashSet<T> = dag.neighbors(b).collect();
    let mut resulting: HashSet<T> = HashSet::new();
    loop {
        if check_a.is_empty() && check_b.is_empty() {
            break;
        }
        if check_a.is_disjoint(&visited_b) && check_b.is_disjoint(&visited_a) {
            let temp: HashSet<T> = check_a
                .iter()
                .cloned()
                .map(|x| dag.neighbors(x))
                .flatten()
                .collect();
            visited_a.extend(check_a);
            check_a = temp;
            let temp: HashSet<T> = check_b
                .iter()
                .cloned()
                .map(|x| dag.neighbors(x))
                .flatten()
                .collect();
            visited_b.extend(check_b);
            check_b = temp;
        } else {
            resulting.extend(check_a.intersection(&visited_b).cloned());
            resulting.extend(check_b.intersection(&visited_a).cloned());
            break;
        }
    }
    resulting.into_iter().map(|x| &dag[&x])
}
