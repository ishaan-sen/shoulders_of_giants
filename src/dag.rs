use crate::Paper;
use std::{ops::IndexMut, rc::Rc};

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
// fn last_common_ancestor(dag: &impl Dag<NodeWeight = Paper>) -> impl Iterator<Item = Paper> {
//     todo!()
// }
