use std::collections::VecDeque;
use std::iter::{IntoIterator, Iterator};
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
    pub fn new(data: &[Vec<Base>]) -> Self {
        let mut dna = VecDeque::new();
        for vec in data {
            dna.extend(vec);
        }
        DNA { dna }
    }

    pub fn len(&self) -> usize {
        self.dna.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dna.is_empty()
    }

    pub fn slice(&self, range: Range<usize>) -> DNASlice {
        let mut parts = vec![];
        let (left, right) = self.dna.as_slices();
        if range.end <= left.len() {
            parts.push(&left[range]);
        } else if range.start >= left.len() {
            if !right.is_empty() {
                let start = range.start - left.len();
                let end = range.end - left.len();
                if start < right.len() {
                    parts.push(&right[start..end.min(right.len())]);
                }
            }
        } else {
            parts.push(&left[range.start..]);
            let end = range.end - left.len();
            if !right.is_empty() {
                parts.push(&right[0..end.min(right.len())]);
            }
        }
        DNASlice { parts }
    }

    pub fn extend_front(&mut self, data: Vec<Base>) {
        for b in data.into_iter().rev() {
            self.dna.push_front(b);
        }
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

pub struct DNASlice<'a> {
    parts: Vec<&'a [Base]>,
}

impl<'a> Index<usize> for DNASlice<'a> {
    type Output = Base;

    fn index(&self, mut idx: usize) -> &Self::Output {
        for p in &self.parts {
            if idx < p.len() {
                return &p[idx];
            }
            idx -= p.len();
        }
        panic!()
    }
}

impl<'a> IntoIterator for DNASlice<'a> {
    type Item = Base;
    type IntoIter = DNASliceIntoIter<'a>;

    fn into_iter(self) -> DNASliceIntoIter<'a> {
        DNASliceIntoIter {
            slice: self,
            idx_part: 0,
            idx: 0,
        }
    }
}

impl<'a> DNASlice<'a> {
    pub fn len(&self) -> usize {
        self.parts.iter().map(|p| p.len()).sum()
    }

    pub fn iter(&self) -> DNASliceIter<'a> {
        DNASliceIter {
            slice: self as *const DNASlice,
            idx_part: 0,
            idx: 0,
        }
    }
}

pub struct DNASliceIntoIter<'a> {
    slice: DNASlice<'a>,
    idx_part: usize,
    idx: usize,
}

impl<'a> Iterator for DNASliceIntoIter<'a> {
    type Item = Base;

    fn next(&mut self) -> Option<Base> {
        let part = self.slice.parts.get(self.idx_part)?;
        let res = part.get(self.idx)?;
        self.idx += 1;
        if self.idx >= part.len() {
            self.idx_part += 1;
            self.idx = 0;
        }
        Some(*res)
    }
}

pub struct DNASliceIter<'a> {
    slice: *const DNASlice<'a>,
    idx_part: usize,
    idx: usize,
}

impl<'a> Iterator for DNASliceIter<'a> {
    type Item = Base;

    fn next(&mut self) -> Option<Base> {
        unsafe {
            let part = (*self.slice).parts.get(self.idx_part)?;
            let res = part.get(self.idx)?;
            self.idx += 1;
            if self.idx >= part.len() {
                self.idx_part += 1;
                self.idx = 0;
            }
            Some(*res)
        }
    }
}
