#![warn(clippy::pedantic)]

#[allow(unused)]
mod ndarray {
    use itertools::Itertools;
    use std::option::IntoIter;

    pub struct Array<T> {
        shape: Vec<usize>,
        data: Vec<T>,
    }
    impl<T: Clone> Array<T> {
        pub fn from_elem(shape: &[usize], elem: T) -> Self {
            Self {
                data: vec![elem; shape.iter().product()],
                shape: shape.into(),
            }
        }
    }
    impl<T: Default> Array<T> {
        pub fn from_default(shape: &[usize], elem: &T) -> Self {
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
    impl<T: std::fmt::Display> Array<T> {
        pub fn print(&self) {
            let mut str_data: Vec<String> = self
                .data
                .iter()
                .map(std::string::ToString::to_string)
                .collect();
            let str_longest = str_data.iter().map(|x| String::len(&x)).max().unwrap_or(0);
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
    impl<T> std::ops::Index<&[usize]> for Array<T> {
        type Output = T;
        fn index(&self, index: &[usize]) -> &T {
            let idx = indices_to_index(index, &self.shape);
            println!("Index: {idx}");
            assert!(idx < self.data.len(), "Index out of bounds.");
            &self.data[idx]
        }
    }
    impl<T: Default> Default for Array<T> {
        fn default() -> Self {
            Self {
                shape: vec![0],
                data: Vec::<T>::default(),
            }
        }
    }
    fn indices_to_index(indices: &[usize], shape: &[usize]) -> usize {
        indices
            .iter()
            .rev()
            .zip(std::iter::once(&1).chain(shape.iter()))
            .map(|(i, &s)| i * s)
            .sum()
    }
}

use super::Node;
use ndarray::Array;

pub struct AdjDAG {
    nodes: Vec<Node>,
    adj: Array<bool>,
}
// impl AdjDAG {
//     fn new(&self, nodes: Vec<Node>) -> Self {
//         Self {}
//     }
// }

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
        assert_eq!(test_array[&[2, 5, 5]], 12);
    }
}
