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
    total_len: usize,
}

impl DNASlice {
    pub fn len(&self) -> usize {
        self.total_len
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
            total_len: range.end - range.start,
        }
    }

    fn new() -> Self {
        DNASlice {
            parts: vec![],
            total_len: 0,
        }
    }

    fn push_front(&mut self, s: DNAStorageSlice) {
        let length = s.length;
        self.parts.push(s);
        self.total_len += length;
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

const CONSOLIDATION_TARGET_SIZE: usize = 1024; // 1KiB

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
                total_len: length,
            },
        }
    }

    pub fn len(&self) -> usize {
        self.dna.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dna.len() == 0
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
        let mut current_slices = vec![];
        for c in data.into_iter().rev() {
            match c {
                DNAChunk::Owned(d) => {
                    if !current_slices.is_empty() {
                        self.extend_front_slices(current_slices);
                        current_slices = vec![];
                    }
                    self.extend_front_owned(d);
                }
                DNAChunk::Slice(s) => {
                    current_slices.push(s);
                }
            }
        }
        if !current_slices.is_empty() {
            self.extend_front_slices(current_slices);
        }
    }

    fn extend_front_owned(&mut self, data: Vec<Base>) {
        let length = data.len();
        self.dna_storage.push(data);
        self.dna.parts.push(DNAStorageSlice {
            idx: self.dna_storage.len() - 1,
            start: 0,
            length,
        });
        self.dna.total_len += length;
    }

    fn extend_front_slices(&mut self, slices: Vec<DNASlice>) {
        // This method will consolidate small slices as new chunks
        if slices.len() == 1 {
            let slice = &slices[0];
            if let &[single_part] = &slice.parts[..] {
                let length = single_part.length;
                self.dna.parts.push(single_part);
                self.dna.total_len += length;
                return;
            }
        }
        let mut new_slice = DNASlice::new();
        for s in slices {
            for p in s.parts {
                if new_slice.total_len + p.length > CONSOLIDATION_TARGET_SIZE {
                    if new_slice.total_len != 0 {
                        self.extend_from_slice(new_slice);
                        new_slice = DNASlice::new();
                    }
                }
                if p.length > CONSOLIDATION_TARGET_SIZE {
                    let length = p.length;
                    self.dna.parts.push(p);
                    self.dna.total_len += length;
                } else {
                    new_slice.push_front(p);
                }
            }
        }
        if new_slice.total_len != 0 {
            self.extend_from_slice(new_slice);
        }
    }

    fn extend_from_slice(&mut self, slice: DNASlice) {
        if slice.parts.len() == 1 {
            let length = slice.parts[0].length;
            self.dna.parts.push(slice.parts[0]);
            self.dna.total_len += length;
        } else {
            self.extend_front_owned(self.render(&slice));
        }
    }

    pub fn truncate_front(&mut self, mut count: usize) {
        while count > 0 {
            let last_idx = self.dna.parts.len() - 1;
            let slice = &mut self.dna.parts[last_idx];
            if slice.length <= count {
                let slice = self.dna.parts.pop().unwrap();
                count -= slice.length;
                self.dna.total_len -= slice.length;
            } else {
                slice.start += count;
                slice.length -= count;
                self.dna.total_len -= count;
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
        self.dna.total_len -= 1;
        Some(b)
    }

    pub fn debug_print(&self) {
        println!("Total slices: {}", self.dna.parts.len());
        if !self.dna.parts.is_empty() {
            let mut min = self.dna.parts[0].length;
            let mut max = min;
            let mut avg = 0;
            let mut used_indices = std::collections::HashSet::new();
            for p in &self.dna.parts {
                if p.length < min {
                    min = p.length;
                }
                if p.length > max {
                    max = p.length;
                }
                avg += p.length;
                used_indices.insert(p.idx);
            }
            avg /= self.dna.parts.len();
            println!("min: {}, max: {}, avg: {}", min, max, avg);
            println!("Total used chunks: {}", used_indices.len());
        }
        println!("Total chunks: {}", self.dna_storage.len());
    }
}
