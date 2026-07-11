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
    nexts: RefCell<HashSet<NodeRef<T>>>,
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
    fn neighbors(&self, node_id: &NodeId<T>) -> impl Iterator<Item = NodeId<T>> {
        node_id.node.upgrade().into_iter().flat_map(|node| {
            node.nexts
                .borrow()
                .iter()
                .map(|node_ref| NodeId::new(self.graph_id, node_ref))
                .collect_vec()
        })
    }

    fn neighbors_back(&self, node_id: &NodeId<T>) -> impl Iterator<Item = NodeId<T>> {
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
        mut func: impl FnMut(&NodeId<T>, &T) -> bool,
    ) -> impl Iterator<Item = NodeId<T>> {
        let mut found = HashSet::new();

        let mut searched = HashSet::<usize>::new();
        let mut to_search = self
            .heads
            .iter()
            .cloned()
            .collect::<std::collections::VecDeque<_>>();
        while let Some(node_ref) = to_search.pop_front() {
            let id = NodeId::new(self.graph_id, &node_ref);
            if func(&id, &self[&id]) {
                found.insert(id);
            }
            let addr = node_ref.addr();
            searched.insert(addr);
            for next in node_ref.nexts.borrow().iter() {
                if !searched.contains(&next.addr()) {
                    to_search.push_back(next.clone());
                }
            }
        }
        found.into_iter()
    }

    fn find_node(&self, mut func: impl FnMut(&NodeId<T>, &T) -> bool) -> Option<NodeId<T>> {
        let mut searched = HashSet::<usize>::new();
        let mut to_search = self
            .heads
            .iter()
            .cloned()
            .collect::<std::collections::VecDeque<_>>();
        while let Some(node_ref) = to_search.pop_front() {
            let id = NodeId::new(self.graph_id, &node_ref);
            if func(&id, &self[&id]) {
                return Some(id);
            }
            let addr = node_ref.addr();
            searched.insert(addr);
            for next in node_ref.nexts.borrow().iter() {
                if !searched.contains(&next.addr()) {
                    to_search.push_back(next.clone());
                }
            }
        }
        None
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

impl<T> LinkedDag<T> {
    /// Insert a node given its weight and its forward neighbors, ignoring any forward
    /// neighbors that do not exist
    pub fn insert_node_lossy<I: std::borrow::Borrow<NodeId<T>>>(
        &mut self,
        weight: T,
        nexts: impl IntoIterator<Item = I>,
    ) -> NodeId<T> {
        let nexts = nexts
            .into_iter()
            .filter(|node_id| node_id.borrow().graph_id == self.graph_id)
            .filter_map(|node_id| node_id.borrow().node.upgrade())
            .map(NodeRef)
            .collect();
        let node = Node {
            nexts: RefCell::new(nexts),
            prevs: RefCell::default(),
            value: UnsafeCell::new(weight),
        };
        let rc = Rc::new(node);
        for next in rc.nexts.borrow().iter() {
            next.prevs.borrow_mut().insert(NodeWeak(Rc::downgrade(&rc)));
            self.heads.remove(next);
        }
        let node_id = NodeId {
            graph_id: self.graph_id,
            node: NodeWeak(Rc::downgrade(&rc)),
        };
        self.heads.insert(NodeRef(rc));
        node_id
    }
}

impl FromIterator<crate::CSVRecord> for LinkedDag<crate::Paper> {
    fn from_iter<T: IntoIterator<Item = crate::CSVRecord>>(iter: T) -> Self {
        use std::collections::HashMap;
        type Id = NodeId<crate::Paper>;

        let metadata_map: HashMap<Rc<str>, (crate::Paper, HashSet<Rc<str>>)> = iter
            .into_iter()
            .map(|rec| {
                (
                    rec.id.clone(),
                    (
                        crate::Paper {
                            id: rec.id,
                            title: rec.title,
                            abstract_text: rec.abstract_text,
                        },
                        rec.references,
                    ),
                )
            })
            .collect();

        let mut nodes = HashMap::<Rc<str>, NodeRef<crate::Paper>>::new();
        for (id, (metadata, _)) in &metadata_map {
            let node = Node {
                nexts: RefCell::default(),
                prevs: RefCell::default(),
                value: UnsafeCell::new(metadata.clone()),
            };
            nodes.insert(Rc::clone(id), NodeRef(Rc::new(node)));
        }
        for (id, (_, refs)) in &metadata_map {
            for ref_id in refs {
                if !metadata_map.contains_key(ref_id) {
                    continue;
                }
                nodes[id].0.nexts.borrow_mut().insert(nodes[ref_id].clone());
                nodes[ref_id]
                    .0
                    .prevs
                    .borrow_mut()
                    .insert(NodeWeak(Rc::downgrade(&nodes[id].0)));
            }
        }
        let non_head_ids: HashSet<Rc<str>> = nodes
            .keys()
            .flat_map(|id| metadata_map[id].1.iter().cloned())
            .collect();
        let heads: HashSet<_> = nodes
            .into_iter()
            .filter(|(id, _)| !non_head_ids.contains(id))
            .map(|(_, node_ref)| node_ref)
            .collect();

        LinkedDag {
            graph_id: rand::random(),
            heads,
        }
    }
}
