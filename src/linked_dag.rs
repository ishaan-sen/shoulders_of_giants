use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
    rc::Rc,
};

pub struct Node<T> {
    nexts: HashMap<usize, Rc<Node<T>>>,
    value: RefCell<T>,
}

impl<T> Deref for Node<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

fn find_impl<'a, T>(
    nexts: &'a HashMap<usize, Rc<Node<T>>>,
    mut f: impl FnMut(&Node<T>) -> bool,
    searched: &mut HashSet<usize>,
) -> Option<&'a Rc<Node<T>>> {
    for next in nexts.values() {
        if f(next) {
            return Some(next);
        }
    }
    for next in nexts.values() {
        let addr = Rc::as_ptr(next) as usize;
        if !searched.contains(&addr)
            && let found @ Some(_) = find_impl(&next.nexts, &mut f, searched)
        {
            return found;
        }
        searched.insert(addr);
    }
    None
}

pub struct Dag<T> {
    heads: HashMap<usize, Rc<Node<T>>>,
}

impl<T> Dag<T> {
    pub fn new() -> Self {
        Self {
            heads: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        value: T,
        nexts: impl IntoIterator<Item = Rc<Node<T>>>,
    ) -> Rc<Node<T>> {
        let nexts = nexts
            .into_iter()
            .map(|next| {
                let addr = Rc::as_ptr(&next) as usize;
                self.heads.remove(&addr);
                (addr, next)
            })
            .collect();
        let node = Node {
            nexts,
            value: RefCell::new(value),
        };
        let rc = Rc::new(node);
        let addr = Rc::as_ptr(&rc) as usize;
        self.heads.insert(addr, Rc::clone(&rc));
        rc
    }

    pub fn find(&self, f: impl FnMut(&Node<T>) -> bool) -> Option<Rc<Node<T>>> {
        find_impl(&self.heads, f, &mut HashSet::new()).map(Rc::clone)
    }
}
