use std::collections::VecDeque;
use std::result::Result;

struct ExecutionState {
    dna: VecDeque<u8>,
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
#[derive(Debug, PartialEq)]
struct Pattern(Vec<PatternItem>);

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.0 {
            use PatternItem::*;
            match item {
                Base(b) => match b {
                    b'I' => write!(f, "I")?,
                    b'C' => write!(f, "C")?,
                    b'F' => write!(f, "F")?,
                    b'P' => write!(f, "P")?,
                    _ => panic!()
                },
                Skip(n) => write!(f, "!{}", n)?,
                Search(s) => {
                    write!(f, "?")?;
                    for b in s {
                        match b {
                            b'I' => write!(f, "I")?,
                            b'C' => write!(f, "C")?,
                            b'F' => write!(f, "F")?,
                            b'P' => write!(f, "P")?,
                            _ => panic!()
                        }
                    }
                }
                GroupOpen => write!(f, "(")?,
                GroupClose => write!(f, ")")?,
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
enum TemplateItem {
    Base(u8),
    Ref(u32, u32),
    Length(u32),
}
#[derive(Debug, PartialEq)]
struct Template(Vec<TemplateItem>);

impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.0 {
            use TemplateItem::*;
            match item {
                Base(b) => match b {
                    b'I' => write!(f, "I")?,
                    b'C' => write!(f, "C")?,
                    b'F' => write!(f, "F")?,
                    b'P' => write!(f, "P")?,
                    _ => panic!()
                },
                Ref(n, l) => write!(f, "\\{}:{}", n, l)?,
                Length(n) => write!(f, "~{}", n)?,
            }
        }
        Ok(())
    }
}

pub fn execute(dna: &[u8]) -> Vec<u8> {
    let mut state = ExecutionState::new(dna);
    state.execute().unwrap_err(); // The implementation can only return an expected "error"
    return state.rna;
}

#[derive(Debug, PartialEq)]
struct EarlyFinish;
type CanFinishEarly<T> = Result<T, EarlyFinish>;

impl ExecutionState {
    fn new(dna: &[u8]) -> Self {
        ExecutionState {
            dna: dna.iter().map(|c| *c).collect(),
            rna: vec![],
        }
    }

    fn execute(&mut self) -> CanFinishEarly<()> {
        let mut i = 0;
        loop {
            let pattern = self.pattern()?;
            let template = self.template()?;
            println!("iteration {}", i);
            println!("pattern: {}", pattern);
            println!("template: {}", template);
            self.match_replace(pattern, template);
            i += 1;
        }
    }

    fn match_replace(&mut self, pattern: Pattern, template: Template) {
        let mut i = 0;
        let mut env = vec![];
        let mut c = vec![];
        for p in pattern.0 {
            use PatternItem::*;
            match p {
                Base(b) => {
                    if self.dna[i] == b {
                        i += 1;
                    } else {
                        return;
                    }
                }
                Skip(n) => {
                    i += n as usize;
                    if i > self.dna.len() {
                        return;
                    }
                }
                Search(s) => {
                    if s.len() > 0 {
                        let mut success = false;
                        for j in i..(self.dna.len() - s.len()) {
                            let mut is_match = true;
                            for k in 0..s.len() {
                                if self.dna[j + k] != s[k] {
                                    is_match = false;
                                    break;
                                }
                            }
                            if is_match {
                                i = j;
                                success = true;
                                break;
                            }
                        }
                        if !success {
                            return;
                        }
                    }
                }
                GroupOpen => {
                    c.push(i);
                }
                GroupClose => {
                    let (left, right) = self.dna.as_slices();
                    let mut result = vec![];
                    let start = *c.last().unwrap();
                    let end = i;
                    let min = |a, b| if a > b { b } else { a };
                    if end <= left.len() {
                        result.extend_from_slice(&left[start..end]);
                    } else if start >= left.len() {
                        if !right.is_empty() {
                            result.extend_from_slice(
                                &right[min(start - left.len(), right.len() - 1)
                                    ..min(end - left.len(), right.len())],
                            );
                        }
                    } else {
                        result.extend_from_slice(&left[start..]);
                        if !right.is_empty() {
                            result
                                .extend_from_slice(&right[0..min(end - result.len(), right.len())]);
                        }
                    }
                    env.push(result);
                    c.pop();
                }
            }
        }
        self.dna = self.dna.split_off(i);
        self.replace(template, &env);
    }

    fn replace(&mut self, template: Template, env: &Vec<Vec<u8>>) {
        let mut result = vec![];
        for t in template.0 {
            use TemplateItem::*;
            match t {
                Base(b) => result.push(b),
                Ref(n, l) => {
                    if (n as usize) < env.len() {
                        result.extend(Self::protect(l, &env[n as usize]));
                    }
                }
                Length(n) => {
                    let length = if (n as usize) < env.len() {
                        env[n as usize].len()
                    } else {
                        0
                    };
                    result.extend(Self::as_nat(length as u32));
                }
            }
        }
        for b in result.into_iter().rev() {
            self.dna.push_front(b);
        }
    }

    fn pattern(&mut self) -> CanFinishEarly<Pattern> {
        let mut pat = vec![];
        let mut level = 0;
        loop {
            match self.dna.pop_front().ok_or(EarlyFinish)? {
                b'C' => pat.push(PatternItem::Base(b'I')),
                b'F' => pat.push(PatternItem::Base(b'C')),
                b'P' => pat.push(PatternItem::Base(b'F')),
                b'I' => match self.dna.pop_front().ok_or(EarlyFinish)? {
                    b'C' => pat.push(PatternItem::Base(b'P')),
                    b'P' => pat.push(PatternItem::Skip(self.nat()?)),
                    b'F' => {
                        self.dna.pop_front(); // Skip an extra base
                        pat.push(PatternItem::Search(self.consts()));
                    }
                    b'I' => match self.dna.pop_front().ok_or(EarlyFinish)? {
                        b'P' => {
                            level += 1;
                            pat.push(PatternItem::GroupOpen);
                        }
                        b'C' | b'F' => {
                            if level == 0 {
                                break Ok(Pattern(pat));
                            }
                            level -= 1;
                            pat.push(PatternItem::GroupClose);
                        }
                        b'I' => {
                            if self.dna.len() < 7 {
                                self.rna.extend(self.dna.clone());
                                self.dna.clear();
                                break Err(EarlyFinish);
                            }
                            for _ in 0..7 {
                                self.rna.push(self.dna.pop_front().unwrap());
                            }
                        }
                        _ => panic!("Invalid DNA string"),
                    },
                    _ => panic!("Invalid DNA string"),
                },
                _ => panic!("Invalid DNA string"),
            }
        }
    }

    fn template(&mut self) -> CanFinishEarly<Template> {
        let mut result = vec![];
        loop {
            match self.dna.pop_front().ok_or(EarlyFinish)? {
                b'C' => result.push(TemplateItem::Base(b'I')),
                b'F' => result.push(TemplateItem::Base(b'C')),
                b'P' => result.push(TemplateItem::Base(b'F')),
                b'I' => match self.dna.pop_front().ok_or(EarlyFinish)? {
                    b'C' => result.push(TemplateItem::Base(b'P')),
                    b'F' | b'P' => {
                        let l = self.nat()?;
                        result.push(TemplateItem::Ref(self.nat()?, l));
                    }
                    b'I' => match self.dna.pop_front().ok_or(EarlyFinish)? {
                        b'C' | b'F' => break Ok(Template(result)),
                        b'P' => result.push(TemplateItem::Length(self.nat()?)),
                        b'I' => {
                            if self.dna.len() < 7 {
                                self.rna.extend(self.dna.clone());
                                self.dna.clear();
                                break Err(EarlyFinish);
                            }
                            for _ in 0..7 {
                                self.rna.push(self.dna.pop_front().unwrap());
                            }
                        }
                        _ => panic!("Invalid DNA string"),
                    },
                    _ => panic!("Invalid DNA string"),
                },
                _ => panic!("Invalid DNA string"),
            }
        }
    }

    fn nat(&mut self) -> CanFinishEarly<u32> {
        let mut stack = vec![];
        loop {
            match self.dna.pop_front().ok_or(EarlyFinish)? {
                b'P' => {
                    let mut result = 0;
                    for x in stack.into_iter().rev() {
                        result = result * 2 + x;
                    }
                    break Ok(result);
                }
                b'I' | b'F' => stack.push(0),
                b'C' => stack.push(1),
                _ => panic!("Invalid DNA string"),
            }
        }
    }

    fn as_nat(mut n: u32) -> Vec<u8> {
        let mut result = vec![];
        while n > 0 {
            if n % 2 == 0 {
                result.push(b'I');
            } else {
                result.push(b'C');
            }
            n /= 2;
        }
        result.push(b'P');
        result
    }

    fn consts(&mut self) -> Vec<u8> {
        let mut result = vec![];
        loop {
            if self.dna.is_empty() {
                break;
            }
            match self.dna[0] {
                b'C' => {
                    self.dna.pop_front();
                    result.push(b'I');
                }
                b'F' => {
                    self.dna.pop_front();
                    result.push(b'C');
                }
                b'P' => {
                    self.dna.pop_front();
                    result.push(b'F');
                }
                b'I' => {
                    if self.dna.is_empty() {
                        break;
                    }
                    match self.dna[1] {
                        b'C' => {
                            self.dna.pop_front();
                            self.dna.pop_front();
                            result.push(b'P');
                        }
                        _ => break,
                    }
                }
                _ => panic!("Invalid DNA string"),
            }
        }
        result
    }

    fn protect(l: u32, d: &Vec<u8>) -> Vec<u8> {
        let mut result = d.clone();
        for _ in 0..l {
            result = Self::quote(&result);
        }
        result
    }

    fn quote(d: &Vec<u8>) -> Vec<u8> {
        let mut result = vec![];
        for b in d {
            match b {
                b'I' => result.push(b'C'),
                b'C' => result.push(b'F'),
                b'F' => result.push(b'P'),
                b'P' => result.extend(b"IC"),
                _ => panic!("Invalid DNA string"),
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
        assert_eq!(
            ExecutionState::new(b"CIIC").pattern(),
            Ok(Pattern(vec![Base(b'I')]))
        );
        assert_eq!(
            ExecutionState::new(b"IIPIPICPIICICIIF").pattern(),
            Ok(Pattern(vec![GroupOpen, Skip(2), GroupClose, Base(b'P')])),
        );
        assert_eq!(
            ExecutionState::new(b"IIPIPICPIICIFCCFPICIIF").pattern(),
            Ok(Pattern(vec![
                GroupOpen,
                Skip(2),
                GroupClose,
                Search(b"ICFP".into_iter().map(|c| *c).collect())
            ])),
        );
    }

    #[test]
    fn test_execute() {
        let mut state = ExecutionState::new(b"IIPIPICPIICICIIFICCIFPPIICCFPC");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(state.dna, b"PICFC");
        state = ExecutionState::new(b"IIPIPICPIICICIIFICCIFCCCPPIICCFPC");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(state.dna, b"PIICCFCFFPC");
        state = ExecutionState::new(b"IIPIPIICPIICIICCIICFCFC");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(state.dna, b"I");
    }
}
