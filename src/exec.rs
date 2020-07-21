use std::error::Error;
use std::result::Result;

struct ExecutionState<'a> {
    dna: &'a [u8],
    rna: Vec<u8>,
}

#[derive(Debug, PartialEq)]
enum PatternItem {
    Base(u8),
    Skip(u32),
    Search(Vec<u8>),
    GroupOpen,
    GroupClose,
}

pub fn execute(dna: Vec<u8>) -> Vec<u8> {
    panic!()
}

#[derive(Debug, PartialEq)]
struct EarlyFinish;
type CanFinishEarly<T> = Result<T, EarlyFinish>;

impl<'a> ExecutionState<'a> {
    fn new(dna: &'a [u8]) -> Self {
        ExecutionState { dna, rna: vec![] }
    }

    fn pattern(&mut self) -> CanFinishEarly<Vec<PatternItem>> {
        let mut pat = vec![];
        let mut level = 0;
        loop {
            match &self.dna {
                &[b'C', rest @ ..] => {
                    self.dna = rest;
                    pat.push(PatternItem::Base(b'I'));
                }
                &[b'F', rest @ ..] => {
                    self.dna = rest;
                    pat.push(PatternItem::Base(b'C'));
                }
                &[b'P', rest @ ..] => {
                    self.dna = rest;
                    pat.push(PatternItem::Base(b'F'));
                }
                &[b'I', b'C', rest @ ..] => {
                    self.dna = rest;
                    pat.push(PatternItem::Base(b'P'));
                }
                &[b'I', b'P', rest @ ..] => {
                    self.dna = rest;
                    pat.push(PatternItem::Skip(self.nat()?));
                }
                &[b'I', b'F', rest @ ..] => {
                    self.dna = rest;
                    pat.push(PatternItem::Search(self.consts()));
                }
                &[b'I', b'I', b'P', rest @ ..] => {
                    self.dna = rest;
                    level += 1;
                    pat.push(PatternItem::GroupOpen);
                }
                &[b'I', b'I', b'C', rest @ ..] | &[b'I', b'I', b'F', rest @ ..] => {
                    self.dna = rest;
                    if level == 0 {
                        break Ok(pat);
                    }
                    level -= 1;
                    pat.push(PatternItem::GroupClose);
                }
                &[b'I', b'I', b'I', rest @ ..] => {
                    if rest.len() < 7 {
                        self.rna.extend(rest);
                        break Err(EarlyFinish);
                    }
                    self.rna.extend(&rest[0..7]);
                    self.dna = &rest[8..];
                }
                _ => break Err(EarlyFinish),
            }
        }
    }

    fn nat(&mut self) -> CanFinishEarly<u32> {
        // TODO: rewrite to avoid recursion
        match &self.dna {
            &[b'P', rest @ ..] => {
                self.dna = rest;
                Ok(0)
            }
            &[b'I', rest @ ..] | &[b'F', rest @ ..] => {
                self.dna = rest;
                Ok(2 * self.nat()?)
            }
            &[b'C', rest @ ..] => {
                self.dna = rest;
                Ok(2 * self.nat()? + 1)
            }
            _ => Err(EarlyFinish),
        }
    }

    fn consts(&mut self) -> Vec<u8> {
        let mut result = vec![];
        loop {
            match &self.dna {
                &[b'C', rest @ ..] => {
                    self.dna = rest;
                    result.push(b'I');
                }
                &[b'F', rest @ ..] => {
                    self.dna = rest;
                    result.push(b'C');
                }
                &[b'P', rest @ ..] => {
                    self.dna = rest;
                    result.push(b'F');
                }
                &[b'I', b'C', rest @ ..] => {
                    self.dna = rest;
                    result.push(b'P');
                }
                _ => break,
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns() {
        use PatternItem::*;
        assert_eq!(ExecutionState::new(b"CIIC").pattern(), Ok(vec![Base(b'I')]));
        assert_eq!(
            ExecutionState::new(b"IIPIPICPIICICIIF").pattern(),
            Ok(vec![GroupOpen, Skip(2), GroupClose, Base(b'P')]),
        );
        assert_eq!(
            ExecutionState::new(b"IIPIPICPIICIFCFPICIIF").pattern(),
            Ok(vec![
                GroupOpen,
                Skip(2),
                GroupClose,
                Search(b"ICFP".into_iter().map(|c| *c).collect())
            ]),
        );
    }
}
