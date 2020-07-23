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

#[derive(Debug, PartialEq, Copy, Clone)]
struct DNAStorageSlice {
    pub idx: usize,    // storage index
    pub start: usize,  // start within the chunk
    pub length: usize, // length of the slice
}

#[derive(Debug, Clone)]
pub struct DNASlice {
    // The slices are stored in reverse as the DNA is modified from the front
    parts: Vec<DNAStorageSlice>,
}

impl DNASlice {
    pub fn len(&self) -> usize {
        // TODO: cache this
        self.parts.iter().map(|p| p.length).sum()
    }

    pub fn slice(&self, range: Range<usize>) -> DNASlice {
        let mut parts = vec![];
        let mut start = range.start;
        let mut length = range.end - range.start;
        for slice in self.parts.iter().rev() {
            if start >= slice.length {
                start -= slice.length;
            } else {
                let slice_length = (slice.length - start).min(length);
                parts.push(DNAStorageSlice {
                    idx: slice.idx,
                    start: slice.start + start,
                    length: slice_length,
                });
                start = 0;
                length -= slice_length;
                if length == 0 {
                    break;
                }
            }
        }
        DNASlice {
            parts: parts.into_iter().rev().collect(),
        }
    }
}

pub enum DNAChunk {
    Owned(Vec<Base>),
    Slice(DNASlice),
}

pub struct DNA {
    dna_storage: Vec<Vec<Base>>,
    dna: DNASlice,
}

impl Index<usize> for DNA {
    type Output = Base;

    fn index(&self, mut idx: usize) -> &Self::Output {
        for slice in self.dna.parts.iter().rev() {
            if idx >= slice.length {
                idx -= slice.length;
            } else {
                return &self.dna_storage[slice.idx][slice.start + idx];
            }
        }
        panic!("Out of bounds");
    }
}

impl DNA {
    pub fn new(data: &[Vec<Base>]) -> Self {
        let mut storage_chunk = vec![];
        for vec in data {
            storage_chunk.extend(vec);
        }
        let length = storage_chunk.len();
        DNA {
            dna_storage: vec![storage_chunk],
            dna: DNASlice {
                parts: vec![DNAStorageSlice {
                    idx: 0,
                    start: 0,
                    length,
                }],
            },
        }
    }

    pub fn len(&self) -> usize {
        self.dna.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dna.parts.is_empty()
    }

    pub fn slice(&self, range: Range<usize>) -> DNASlice {
        self.dna.slice(range)
    }

    pub fn render(&self, slice: &DNASlice) -> Vec<Base> {
        let mut result = vec![];
        for p in slice.parts.iter().rev() {
            result.extend(&self.dna_storage[p.idx][p.start..(p.start + p.length)]);
        }
        result
    }

    pub fn extend_front(&mut self, data: Vec<DNAChunk>) {
        for c in data.into_iter().rev() {
            match c {
                DNAChunk::Owned(d) => {
                    let length = d.len();
                    self.dna_storage.push(d);
                    self.dna.parts.push(DNAStorageSlice {
                        idx: self.dna_storage.len() - 1,
                        start: 0,
                        length,
                    })
                }
                DNAChunk::Slice(s) => {
                    self.dna.parts.extend(s.parts);
                }
            }
        }
    }

    pub fn truncate_front(&mut self, mut count: usize) {
        while count > 0 {
            let last_idx = self.dna.parts.len() - 1;
            let slice = &mut self.dna.parts[last_idx];
            if slice.length <= count {
                let slice = self.dna.parts.pop().unwrap();
                count -= slice.length;
            } else {
                slice.start += count;
                slice.length -= count;
                break;
            }
        }
    }

    pub fn pop_front(&mut self) -> Option<Base> {
        if self.dna.parts.is_empty() {
            return None;
        }
        let last_idx = self.dna.parts.len() - 1;
        let slice = &mut self.dna.parts[last_idx];
        let b = self.dna_storage[slice.idx][slice.start];
        if slice.length > 1 {
            slice.start += 1;
            slice.length -= 1;
        } else {
            self.dna.parts.pop().unwrap();
        }
        Some(b)
    }
}
