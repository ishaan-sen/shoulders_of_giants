#![allow(
    clippy::mutable_key_type,
    reason = "False positive. NodeRef hashes and compares based purely on the pointer, and not on the content, which is the only part with interior mutability"
)]

use std::{
    cell::{RefCell, UnsafeCell},
    collections::HashSet,
    hash::Hash,
    ops::Deref,
    rc::{Rc, Weak},
};

use itertools::Itertools;

use crate::dag::Dag;

type GraphId = u64;

struct NodeRef<T>(Rc<Node<T>>);
impl<T> NodeRef<T> {
    fn addr(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}
impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
impl<T> PartialEq for NodeRef<T> {
    fn eq(&self, other: &Self) -> bool {
        self.addr() == other.addr()
    }
}
impl<T> Eq for NodeRef<T> {}
impl<T> Hash for NodeRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr().hash(state);
    }
}
impl<T> Deref for NodeRef<T> {
    type Target = Rc<Node<T>>;
    fn deref(&self) -> &Rc<Node<T>> {
        &self.0
    }
}

struct NodeWeak<T>(Weak<Node<T>>);
impl<T> NodeWeak<T> {
    fn addr(&self) -> usize {
        Weak::as_ptr(&self.0) as usize
    }
}
impl<T> Clone for NodeWeak<T> {
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}
impl<T> PartialEq for NodeWeak<T> {
    fn eq(&self, other: &Self) -> bool {
        self.addr() == other.addr()
    }
}
impl<T> Eq for NodeWeak<T> {}
impl<T> Hash for NodeWeak<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.addr().hash(state);
    }
}
impl<T> Deref for NodeWeak<T> {
    type Target = Weak<Node<T>>;
    fn deref(&self) -> &Weak<Node<T>> {
        &self.0
    }
}

struct Node<T> {
    nexts: HashSet<NodeRef<T>>,
    prevs: RefCell<HashSet<NodeWeak<T>>>,
    value: UnsafeCell<T>,
}

pub struct NodeId<T> {
    graph_id: GraphId,
    node: NodeWeak<T>,
}
impl<T> NodeId<T> {
    fn new(graph_id: GraphId, rc: &Rc<Node<T>>) -> Self {
        Self {
            graph_id,
            node: NodeWeak(Rc::downgrade(rc)),
        }
    }
}
impl<T> Clone for NodeId<T> {
    fn clone(&self) -> Self {
        Self {
            graph_id: self.graph_id,
            node: self.node.clone(),
        }
    }
}
impl<T> PartialEq for NodeId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}
impl<T> Eq for NodeId<T> {}
impl<T> Hash for NodeId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.node.hash(state);
    }
}

pub struct LinkedDag<T> {
    graph_id: GraphId,
    heads: HashSet<NodeRef<T>>,
}

impl<T> Default for LinkedDag<T> {
    fn default() -> Self {
        Self {
            graph_id: rand::random(),
            heads: HashSet::new(),
        }
    }
}

impl<T> std::ops::Index<&NodeId<T>> for LinkedDag<T> {
    type Output = T;
    fn index(&self, index: &NodeId<T>) -> &T {
        self.get(index).expect("NodeId does not exist in graph")
    }
}
impl<T> std::ops::IndexMut<&NodeId<T>> for LinkedDag<T> {
    fn index_mut(&mut self, index: &NodeId<T>) -> &mut T {
        self.get_mut(index).expect("NodeId does not exist in graph")
    }
}

impl<T> Dag for LinkedDag<T> {
    type NodeWeight = T;
    type NodeId = NodeId<T>;
    fn neighbors(&self, node_id: NodeId<T>) -> impl Iterator<Item = NodeId<T>> {
        node_id.node.upgrade().into_iter().flat_map(|node| {
            node.nexts
                .iter()
                .map(|node_ref| NodeId::new(self.graph_id, node_ref))
                .collect_vec()
        })
    }

    fn neighbors_back(&self, node_id: NodeId<T>) -> impl Iterator<Item = NodeId<T>> {
        node_id.node.upgrade().into_iter().flat_map(|node| {
            node.prevs
                .borrow()
                .iter()
                .filter_map(|node_weak| {
                    node_weak
                        .upgrade()
                        .map(|node| NodeId::new(self.graph_id, &node))
                })
                .collect_vec()
        })
    }

    fn find_nodes(
        &self,
        func: impl FnMut(&NodeId<T>, &T) -> bool,
    ) -> impl Iterator<Item = NodeId<T>> {
        let mut found = HashSet::new();
        find_nodes_impl(self, &self.heads, func, &mut found, &mut HashSet::new());
        found.into_iter()
    }

    fn find_node(&self, func: impl FnMut(&NodeId<T>, &T) -> bool) -> Option<NodeId<T>> {
        find_node_impl(self, &self.heads, func, &mut HashSet::new())
    }

    fn get(&self, id: &NodeId<T>) -> Option<&T> {
        // SAFETY: Assuming that this is the only `LinkedDag` with this `graph_id`, this
        // implementation causes `self` to act like it owns the value contained in the
        // `UnsafeCell`, meaning that the borrow checker will not allow borrow rules
        // to be violated when accessing the inner value.
        if id.graph_id != self.graph_id {
            return None;
        }
        id.node.upgrade().map(|node| unsafe { &*node.value.get() })
    }

    fn get_mut(&mut self, id: &NodeId<T>) -> Option<&mut T> {
        // SAFETY: See `get` safety comment.
        if id.graph_id != self.graph_id {
            return None;
        }
        id.node
            .upgrade()
            .map(|node| unsafe { &mut *node.value.get() })
    }
}

fn find_nodes_impl<T>(
    graph: &LinkedDag<T>,
    nexts: &HashSet<NodeRef<T>>,
    mut func: impl FnMut(&NodeId<T>, &T) -> bool,
    found: &mut HashSet<NodeId<T>>,
    searched: &mut HashSet<usize>,
) {
    for next in nexts {
        let id = NodeId::new(graph.graph_id, next);
        if func(&id, &graph[&id]) {
            found.insert(id);
        }
    }
    for next in nexts {
        let addr = next.addr();
        if !searched.contains(&addr) {
            find_nodes_impl(graph, &next.nexts, &mut func, found, searched);
        }
        searched.insert(addr);
    }
}

fn find_node_impl<T>(
    graph: &LinkedDag<T>,
    nexts: &HashSet<NodeRef<T>>,
    mut func: impl FnMut(&NodeId<T>, &T) -> bool,
    searched: &mut HashSet<usize>,
) -> Option<NodeId<T>> {
    for next in nexts {
        let id = NodeId::new(graph.graph_id, next);
        if func(&id, &graph[&id]) {
            return Some(id);
        }
    }
    for next in nexts {
        let addr = next.addr();
        if !searched.contains(&addr)
            && let found @ Some(_) = find_node_impl(graph, &next.nexts, &mut func, searched)
        {
            return found;
        }
        searched.insert(addr);
    }
    None
}
