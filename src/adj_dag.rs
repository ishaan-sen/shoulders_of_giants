#![warn(clippy::pedantic)]

#[allow(unused)]
// This custom ndarray implementation is used as the backend for the adjacency matrix. In order to avoid numerous reallocations during construction, all constructors allocate in one shot.
mod ndarray {
    use itertools::Itertools;
    use std::option::IntoIter;

    pub struct Array<T> {
        shape: Vec<usize>,
        data: Vec<T>,
    }
    // Create an array by cloning a single element into every position
    impl<T: Clone> Array<T> {
        pub fn from_elem(shape: &[usize], elem: T) -> Self {
            Self {
                data: vec![elem; shape.iter().product()],
                shape: shape.into(),
            }
        }
    }
    // Create an array by initializing every element with the default value for that type
    impl<T: Default> Array<T> {
        pub fn from_default(shape: &[usize]) -> Self {
            let elem_ct = shape.iter().product();
            let mut data = Vec::with_capacity(elem_ct);
            for i in 0..elem_ct {
                data.push(T::default());
            }
            Self {
                data,
                shape: shape.into(),
            }
        }
    }
    // Create an array by filling indices by iterator
    impl<T> Array<T> {
        pub fn from_iter<I: Iterator<Item = T>>(shape: &[usize], iter: I) -> Self {
            let data: Vec<T> = iter.collect();
            assert!(
                data.len() >= shape.iter().product(),
                "Iterator is too short for this shape."
            );
            Self {
                data,
                shape: shape.into(),
            }
        }
        // Create an array by filling indices with the output of a function of the indices
        pub fn from_fn<I: FromIterator<usize>, F: Fn(I) -> T>(shape: &[usize], func: F) -> Self {
            let elem_ct = shape.iter().product();
            let mut data = Vec::with_capacity(elem_ct);
            shape
                .iter()
                .map(|x| 0..*x)
                .multi_cartesian_product()
                .for_each(|vec| data.push(func(vec.into_iter().collect())));
            Self {
                data,
                shape: shape.into(),
            }
        }
    }
    // Print an array to the console, with the last axis being horizontal, and the remaining axes printed vertically, with increasing spacing
    impl<T: std::fmt::Display> Array<T> {
        pub fn print(&self) {
            let mut str_data: Vec<String> = self
                .data
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            let str_longest = str_data.iter().map(String::len).max().unwrap_or(0);
            str_data = str_data
                .iter()
                .map(|x| format!("{x:>str_longest$} "))
                .collect();
            str_data.iter().enumerate().for_each(|(i, x)| {
                self.shape
                    .iter()
                    .skip(1)
                    .rev()
                    .scan(1_usize, |a, b| {
                        *a *= b;
                        Some(*a)
                    })
                    .for_each(|s| {
                        if i % s == 0 && i != 0 {
                            println!();
                        }
                    });
                print!("{x}");
            });
        }
    }
    // Acquire a reference to the value at an index
    impl<T> std::ops::Index<&[usize]> for Array<T> {
        type Output = T;
        fn index(&self, index: &[usize]) -> &T {
            let idx = indices_to_index(index, &self.shape);
            assert!(idx < self.data.len(), "Index out of bounds.");
            &self.data[idx]
        }
    }
    // Acquire a mutable reference to the value at an index
    impl<T> std::ops::IndexMut<&[usize]> for Array<T> {
        fn index_mut(&mut self, index: &[usize]) -> &mut T {
            let idx = indices_to_index(index, &self.shape);
            assert!(idx < self.data.len(), "Index out of bounds.");
            &mut self.data[idx]
        }
    }
    // Create a default, empty ndarray
    impl<T: Default> Default for Array<T> {
        fn default() -> Self {
            Self {
                shape: vec![0],
                data: Vec::<T>::default(),
            }
        }
    }
    // Convert from a shaped index to a flat index
    fn indices_to_index(indices: &[usize], shape: &[usize]) -> usize {
        indices
            .iter()
            .rev()
            .zip(
                std::iter::once(&1_usize)
                    .chain(shape.iter().skip(1).rev())
                    .scan(1, |a, b| {
                        *a *= b;
                        Some(*a)
                    }),
            )
            .map(|(i, s)| *i * s)
            .sum()
    }
}

use core::ops::Index;
use std::collections::HashMap;
use std::iter::Iterator;
use std::ops::IndexMut;
use std::rc::Rc;

use super::CSVRecord;
use super::Paper;
use super::dag::Dag;
use ndarray::Array;

// DAG holds a list of the nodes, as well as a matrix describing the connections
// Note that the memory footprint of the matrix increases with the square of the node count
pub struct AdjDag<T> {
    pub nodes: Vec<T>,
    adj: Array<bool>,
}
// The general way to construct a dag across the implementations is from an iterator
impl FromIterator<CSVRecord> for AdjDag<Paper> {
    fn from_iter<T: IntoIterator<Item = CSVRecord>>(rec_iter: T) -> Self {
        let records: Vec<CSVRecord> = rec_iter.into_iter().collect();
        let indexmap: HashMap<Rc<str>, usize> = records
            .iter()
            .map(|x| x.id.clone())
            .enumerate()
            .map(|(a, b)| (b, a))
            .collect();
        let mut adj: Array<bool> = Array::from_default(&[indexmap.len(), indexmap.len()]);
        for rec in &records {
            for rf in &rec.references {
                if let Some(&b) = indexmap.get(rf) {
                    adj[&[indexmap[&rec.id], b]] = true;
                }
            }
        }
        let nodes = records
            .iter()
            .map(|r| Paper {
                id: r.id.clone(),
                title: r.title.clone(),
                abstract_text: r.abstract_text.clone(),
            })
            .collect();
        Self { nodes, adj }
    }
}
// Implementations of the standard DAG interface
impl<T> Dag for AdjDag<T> {
    type NodeWeight = T;

    type NodeId = usize;

    // Return an iterator of node ids filtered by whether there is true or false in the matrix column
    fn neighbors(&self, node: &Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        (0..self.nodes.len()).filter(|connection| self.adj[&[*node, *connection]])
    }
    // Return an iterator of node ids filtered by whether there is a true or false in the matrix row
    fn neighbors_back(&self, node: &Self::NodeId) -> impl Iterator<Item = Self::NodeId> {
        (0..self.nodes.len()).filter(|connection| self.adj[&[*connection, *node]])
    }
    // Return an iterator of nodes filtered by a custom function
    fn find_nodes(
        &self,
        mut func: impl FnMut(&Self::NodeId, &Self::NodeWeight) -> bool,
    ) -> impl Iterator<Item = Self::NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(move |(id, node)| if func(&id, node) { Some(id) } else { None })
    }
    // Return the first node, if any, matching a custom function
    fn find_node(
        &self,
        mut func: impl FnMut(&Self::NodeId, &Self::NodeWeight) -> bool,
    ) -> Option<Self::NodeId> {
        self.nodes
            .iter()
            .enumerate()
            .find_map(|(id, node)| if func(&id, node) { Some(id) } else { None })
    }
    // Fallible versions of the indexing functions, with bounds checking
    fn get(&self, id: &Self::NodeId) -> Option<&Self::NodeWeight> {
        if *id < self.nodes.len() {
            Some(&self.nodes[*id])
        } else {
            None
        }
    }
    fn get_mut(&mut self, id: &Self::NodeId) -> Option<&mut Self::NodeWeight> {
        if *id < self.nodes.len() {
            Some(&mut self.nodes[*id])
        } else {
            None
        }
    }
}
impl<T> Index<&usize> for AdjDag<T> {
    type Output = T;
    fn index(&self, index: &usize) -> &Self::Output {
        &self.nodes[*index]
    }
}
impl<T> IndexMut<&usize> for AdjDag<T> {
    fn index_mut(&mut self, index: &usize) -> &mut Self::Output {
        &mut self.nodes[*index]
    }
}

// These were used during testing to make sure that arrays and DAGs were being constructed correctly
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_from_fn() {
        let test_array: Array<i32> = Array::from_fn(&[2, 10, 10], |x: Vec<usize>| {
            i32::try_from(x.iter().sum::<usize>()).unwrap_or(0)
        });
        test_array.print();
    }
    #[test]
    fn test_index() {
        let test_array: Array<i32> = Array::from_fn(&[2, 10, 10], |x: Vec<usize>| {
            i32::try_from(x.iter().sum::<usize>()).unwrap_or(0)
        });
        assert_eq!(test_array[&[1, 5, 5]], 11);
    }
    #[test]
    fn test_dag() {
        let records: [CSVRecord; 3] = [
            CSVRecord {
                id: "a".into(),
                title: "".into(),
                abstract_text: "".into(),
                references: std::collections::HashSet::default(),
            },
            CSVRecord {
                id: "b".into(),
                title: "".into(),
                abstract_text: "".into(),
                references: std::collections::HashSet::from(["a".into()]),
            },
            CSVRecord {
                id: "c".into(),
                title: "".into(),
                abstract_text: "".into(),
                references: std::collections::HashSet::from(["a".into(), "b".into()]),
            },
        ];
        let test_dag: AdjDag<Paper> = records.into_iter().collect();
        test_dag.adj.print();
    }
}
