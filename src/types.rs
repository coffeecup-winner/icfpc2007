use std::collections::VecDeque;
use std::ops::{Index, Range};

pub use Base::*;

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Base {
    I = b'I',
    C = b'C',
    F = b'F',
    P = b'P',
}

impl std::fmt::Display for Base {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            I => write!(f, "I"),
            C => write!(f, "C"),
            F => write!(f, "F"),
            P => write!(f, "P"),
        }
    }
}

// Panics if `data` contains an invalid base
pub fn to_base_vec(data: &[u8]) -> Vec<Base> {
    data.into_iter()
        .map(|c| match c {
            b'I' => I,
            b'C' => C,
            b'F' => F,
            b'P' => P,
            _ => panic!(),
        })
        .collect()
}

pub fn to_u8_vec(bases: &[Base]) -> Vec<u8> {
    bases
        .into_iter()
        .map(|b| match b {
            I => b'I',
            C => b'C',
            F => b'F',
            P => b'P',
        })
        .collect()
}

pub struct DNA {
    dna: VecDeque<Base>,
}

impl Index<usize> for DNA {
    type Output = Base;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.dna[idx]
    }
}

impl DNA {
    pub fn new() -> Self {
        DNA {
            dna: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.dna.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dna.is_empty()
    }

    pub fn slice(&self, range: Range<usize>) -> Vec<Base> {
        let mut result = vec![];
        let (left, right) = self.dna.as_slices();
        if range.end <= left.len() {
            result.extend_from_slice(&left[range]);
        } else if range.start >= left.len() {
            if !right.is_empty() {
                let start = range.start - left.len();
                let end = range.end - left.len();
                if start < right.len() {
                    result.extend_from_slice(&right[start..end.min(right.len())]);
                }
            }
        } else {
            result.extend_from_slice(&left[range.start..]);
            let end = range.end - left.len();
            if !right.is_empty() {
                result.extend_from_slice(&right[0..end.min(right.len())]);
            }
        }
        result
    }

    pub fn extend_front(&mut self, data: Vec<Base>) {
        for b in data.into_iter().rev() {
            self.dna.push_front(b);
        }
    }

    pub fn extend_back(&mut self, data: Vec<Base>) {
        self.dna.extend(&data);
    }

    pub fn truncate_front(&mut self, count: usize) {
        if count <= self.dna.len() {
            self.dna = self.dna.split_off(count);
        }
    }

    pub fn pop_front(&mut self) -> Option<Base> {
        self.dna.pop_front()
    }
}
