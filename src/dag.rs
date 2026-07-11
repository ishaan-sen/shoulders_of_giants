use crate::Paper;
use std::{collections::HashSet, ops::IndexMut};

pub trait Dag: for<'id> IndexMut<&'id Self::NodeId, Output = Self::NodeWeight> {
    type NodeWeight;
    type NodeId: Eq + std::hash::Hash + Clone;
    /// Return forward-neighbors of this node in no particular order
    fn neighbors(&self, node: &Self::NodeId) -> impl Iterator<Item = Self::NodeId>;
    /// Return reverse-neighbors of this node in no particular order
    fn neighbors_back(&self, node: &Self::NodeId) -> impl Iterator<Item = Self::NodeId>;
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

pub fn find_by_id<'g>(dag: &'g impl Dag<NodeWeight = Paper>, id: &str) -> Option<&'g Paper> {
    dag.find_node(|_, weight| *weight.id == *id)
        .map(|node_id| &dag[&node_id])
}

// fn earliest_common_descendant(dag: &impl Dag<NodeWeight = Paper>) -> impl Iterator<Item = Paper> {
//     todo!()
// }
pub fn last_common_ancestor<'g, G: Dag<NodeWeight = Paper>>(
    dag: &'g G,
    a: &G::NodeId,
    b: &G::NodeId,
) -> impl Iterator<Item = &'g Paper> {
    let mut visited_a = HashSet::<G::NodeId>::new();
    let mut visited_b = HashSet::<G::NodeId>::new();
    let mut check_a: HashSet<G::NodeId> = dag.neighbors(a).collect();
    let mut check_b: HashSet<G::NodeId> = dag.neighbors(b).collect();
    let mut resulting = HashSet::<G::NodeId>::new();
    loop {
        if check_a.is_empty() && check_b.is_empty() {
            break;
        }

        let meet: HashSet<G::NodeId> = check_a.intersection(&check_b).cloned().collect();
        if !meet.is_empty() {
            resulting.extend(meet);
            break;
        }

        if check_a.is_disjoint(&visited_b) && check_b.is_disjoint(&visited_a) {
            let temp: HashSet<G::NodeId> = check_a.iter().flat_map(|x| dag.neighbors(x)).collect();
            visited_a.extend(check_a);
            check_a = temp;
            let temp: HashSet<G::NodeId> = check_b.iter().flat_map(|x| dag.neighbors(x)).collect();
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
