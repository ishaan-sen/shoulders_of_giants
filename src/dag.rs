use std::ops::IndexMut;

pub trait Dag: Default + IndexMut<Self::NodeId, Output = Self::NodeWeight> {
    type NodeWeight;
    type NodeId;
    /// Return forward-neighbors of this node in no particular order
    fn neighbors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;
    fn find_nodes(
        &self,
        func: impl FnMut(Self::NodeId, &Self::NodeWeight) -> bool,
    ) -> impl Iterator<Item = Self::NodeId>;
    fn find_node(
        &self,
        func: impl FnMut(Self::NodeId, &Self::NodeWeight) -> bool,
    ) -> Option<Self::NodeId>;
    fn get(&self, id: Self::NodeId) -> Option<&Self::NodeWeight>;
    fn get_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight>;
    // what other functions go here?
}
