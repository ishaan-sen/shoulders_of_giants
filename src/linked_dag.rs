#![allow(
    clippy::mutable_key_type,
    reason = "False positive. NodeRef hashes and compares based purely on the pointer, and not on the content, which is the only part with interior mutability"
)]

use std::{
    cell::RefCell,
    collections::HashSet,
    hash::Hash,
    ops::Deref,
    rc::{Rc, Weak},
};

type GraphId = u64;

pub struct NodeId<T> {
    graph_id: GraphId,
    node: Weak<Node<T>>,
}
impl<T> NodeId<T> {
    fn new(graph_id: GraphId, rc: &Rc<Node<T>>) -> Self {
        Self {
            graph_id,
            node: Rc::downgrade(rc),
        }
    }
}

struct NodeRef<T> {
    node: Rc<Node<T>>,
}

impl<T> NodeRef<T> {
    fn addr(&self) -> usize {
        Rc::as_ptr(&self.node) as usize
    }
}
impl<T> From<&Rc<Node<T>>> for NodeRef<T> {
    fn from(value: &Rc<Node<T>>) -> Self {
        Self {
            node: Rc::clone(value),
        }
    }
}
impl<T> Clone for NodeRef<T> {
    fn clone(&self) -> Self {
        Self {
            node: Rc::clone(&self.node),
        }
    }
}
impl<T> PartialEq for NodeRef<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::as_ptr(&self.node) == Rc::as_ptr(&other.node)
    }
}
impl<T> Eq for NodeRef<T> {}
impl<T> Hash for NodeRef<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.node).hash(state);
    }
}
// impl<T> Deref for NodeRef<T> {
//     type Target = RefCell<T>;
//     fn deref(&self) -> &Self::Target {
//         &self.node.value
//     }
// }

struct Node<T> {
    nexts: HashSet<NodeRef<T>>,
    value: RefCell<T>,
}

// fn find_impl<'a, T>(
//     nexts: &'a HashSet<NodeRef<T>>,
//     mut f: impl FnMut(&Node<T>) -> bool,
//     searched: &mut HashSet<usize>,
// ) -> Option<&'a NodeRef<T>> {
//     for next in nexts {
//         if f(&next.node) {
//             return Some(next);
//         }
//     }
//     for next in nexts {
//         let addr = next.addr();
//         if !searched.contains(&addr)
//             && let found @ Some(_) = find_impl(&next.node.nexts, &mut f, searched)
//         {
//             return found;
//         }
//         searched.insert(addr);
//     }
//     None
// }

pub struct LinkedDag<T> {
    graph_id: GraphId,
    heads: HashSet<NodeRef<T>>,
}

// impl<T, F> FromIterator<(T, F)> for LinkedDag<T> where F: FnMut(NodeId<T>, &T) {}

// fn hi() {
//     let ns = vec![1, 2, 3];
//     let fs: Vec<_> = ns.into_iter().map(|x| move |y| x == y).collect();
// }

impl<T> LinkedDag<T> {
    pub fn new() -> Self {
        Self {
            graph_id: rand::random(),
            heads: HashSet::new(),
        }
    }

    // pub fn neighbors(&self, node_id: NodeId<T>) -> impl IntoIterator<Item = NodeId<T>> {}

    // pub fn insert(&mut self, value: T, nexts: impl IntoIterator<Item = NodeId<T>>) -> NodeId<T> {
    //     let nexts = nexts
    //         .into_iter()
    //         .inspect(|next| {
    //             self.heads.remove(&next);
    //         })
    //         .collect();
    //     let node = Node {
    //         nexts,
    //         value: RefCell::new(value),
    //     };
    //     let rc = Rc::new(node);
    //     let node_id = NodeId::new(self.graph_id, &rc);
    //     let node_ref = NodeRef { node: rc };
    //     // self.heads.insert(node_ref.clone());
    //     self.heads.insert(node_ref);
    //     node_id
    // }

    // pub fn find(&self, f: impl FnMut(&Node<T>) -> bool) -> Option<NodeRef<T>> {
    //     find_impl(&self.heads, f, &mut HashSet::new()).cloned()
    // }
}
