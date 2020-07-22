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
