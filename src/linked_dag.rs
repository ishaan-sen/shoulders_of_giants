#![allow(
    clippy::mutable_key_type,
    reason = "False positive. NodeRef hashes and compares based purely on the pointer, and not on the content, which is the only part with interior mutability"
)]

use std::{cell::RefCell, collections::HashSet, hash::Hash, ops::Deref, rc::Rc};

pub struct NodeRef<T> {
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
impl<T> Deref for NodeRef<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &Self::Target {
        &self.node.value
    }
}

struct Node<T> {
    nexts: HashSet<NodeRef<T>>,
    value: RefCell<T>,
}

fn find_impl<'a, T>(
    nexts: &'a HashSet<NodeRef<T>>,
    mut f: impl FnMut(&NodeRef<T>) -> bool,
    searched: &mut HashSet<usize>,
) -> Option<&'a NodeRef<T>> {
    for next in nexts {
        if f(next) {
            return Some(next);
        }
    }
    for next in nexts {
        let addr = next.addr();
        if !searched.contains(&addr)
            && let found @ Some(_) = find_impl(&next.node.nexts, &mut f, searched)
        {
            return found;
        }
        searched.insert(addr);
    }
    None
}

pub struct Dag<T> {
    heads: HashSet<NodeRef<T>>,
}

impl<T> Dag<T> {
    pub fn new() -> Self {
        Self {
            heads: HashSet::new(),
        }
    }

    pub fn insert(&mut self, value: T, nexts: impl IntoIterator<Item = NodeRef<T>>) -> NodeRef<T> {
        let nexts = nexts
            .into_iter()
            .inspect(|next| {
                self.heads.remove(&next);
            })
            .collect();
        let node = Node {
            nexts,
            value: RefCell::new(value),
        };
        let rc = Rc::new(node);
        let node_ref = NodeRef { node: rc };
        self.heads.insert(node_ref.clone());
        node_ref
    }

    pub fn find(&self, f: impl FnMut(&NodeRef<T>) -> bool) -> Option<NodeRef<T>> {
        find_impl(&self.heads, f, &mut HashSet::new()).cloned()
    }
}
