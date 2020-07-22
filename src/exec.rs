use std::collections::VecDeque;
use std::result::Result;

use crate::types::*;

struct ExecutionState {
    dna: VecDeque<Base>,
    rna: Vec<Base>,
}

#[derive(Debug, PartialEq)]
enum PatternItem {
    Base(Base),
    Skip(u32),
    Search(Vec<Base>),
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
                Base(b) => write!(f, "{:?}", b)?,
                Skip(n) => write!(f, "!{}", n)?,
                Search(s) => {
                    write!(f, "?\"")?;
                    for b in s {
                        write!(f, "{:?}", b)?;
                    }
                    write!(f, "\"")?;
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
    Base(Base),
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
                Base(b) => write!(f, "{:?}", b)?,
                Ref(n, l) => write!(f, "\\{}:{}", n, l)?,
                Length(n) => write!(f, "~{}", n)?,
            }
        }
        Ok(())
    }
}

pub fn execute(prefix: &[u8], dna: &[u8]) -> Vec<Base> {
    let mut state = ExecutionState::new(prefix, dna);
    state.execute().unwrap_err(); // The implementation can only return an expected "error"
    return state.rna;
}

#[derive(Debug, PartialEq)]
struct EarlyFinish;
type CanFinishEarly<T> = Result<T, EarlyFinish>;

impl ExecutionState {
    fn new(prefix: &[u8], dna: &[u8]) -> Self {
        let mut dna_deque = VecDeque::with_capacity(prefix.len() + dna.len());
        dna_deque.extend(to_base_vec(prefix));
        dna_deque.extend(to_base_vec(dna));
        ExecutionState {
            dna: dna_deque,
            rna: vec![],
        }
    }

    fn execute(&mut self) -> CanFinishEarly<()> {
        let mut i = 0;
        loop {
            println!("iteration {}", i);
            println!("dna length: {}", self.dna.len());
            let pattern = self.pattern()?;
            let template = self.template()?;
            println!("pattern: {}", pattern);
            println!("template: {}", template);
            self.match_replace(pattern, template);
            println!("rna length: {} ({})", self.rna.len() / 7, self.rna.len());
            println!();
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
                                i = j + s.len();
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
                    // TODO: fix the below
                    //
                    if end <= left.len() {
                        result.extend_from_slice(&left[start..end]);
                    } else if start >= left.len() {
                        if !right.is_empty() {
                            let start = start - left.len();
                            let end = end - left.len();
                            if start < left.len() {
                                result.extend_from_slice(&right[start..end.min(right.len())]);
                            }
                        }
                    } else {
                        result.extend_from_slice(&left[start..]);
                        let end = end - left.len();
                        if !right.is_empty() {
                            result.extend_from_slice(&right[0..end.min(right.len())]);
                        }
                    }
                    // let mut data = vec![];
                    // data.extend_from_slice(left);
                    // data.extend_from_slice(right);
                    // result.extend_from_slice(&data[start..end]);
                    env.push(result);
                    c.pop();
                }
            }
        }
        println!("match length: {}", i);
        self.dna = self.dna.split_off(i);
        self.replace(template, &env);
    }

    fn replace(&mut self, template: Template, env: &Vec<Vec<Base>>) {
        let print = |&b| match b {
            I => 'I',
            C => 'C',
            F => 'F',
            P => 'P',
        };
        for (i, p) in env.iter().enumerate() {
            println!(
                "env[{}] = {}{} ({})",
                i,
                &p[0..10.min(p.len())].iter().map(print).collect::<String>(),
                if p.len() > 10 { "..." } else { "" },
                p.len()
            );
        }
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
                C => pat.push(PatternItem::Base(I)),
                F => pat.push(PatternItem::Base(C)),
                P => pat.push(PatternItem::Base(F)),
                I => match self.dna.pop_front().ok_or(EarlyFinish)? {
                    C => pat.push(PatternItem::Base(P)),
                    P => pat.push(PatternItem::Skip(self.nat()?)),
                    F => {
                        self.dna.pop_front(); // Skip an extra base
                        pat.push(PatternItem::Search(self.consts()));
                    }
                    I => match self.dna.pop_front().ok_or(EarlyFinish)? {
                        P => {
                            level += 1;
                            pat.push(PatternItem::GroupOpen);
                        }
                        C | F => {
                            if level == 0 {
                                break Ok(Pattern(pat));
                            }
                            level -= 1;
                            pat.push(PatternItem::GroupClose);
                        }
                        I => {
                            if self.dna.len() < 7 {
                                self.rna.extend(self.dna.clone());
                                self.dna.clear();
                                break Err(EarlyFinish);
                            }
                            for _ in 0..7 {
                                self.rna.push(self.dna.pop_front().unwrap());
                            }
                        }
                    },
                },
            }
        }
    }

    fn template(&mut self) -> CanFinishEarly<Template> {
        let mut result = vec![];
        loop {
            match self.dna.pop_front().ok_or(EarlyFinish)? {
                C => result.push(TemplateItem::Base(I)),
                F => result.push(TemplateItem::Base(C)),
                P => result.push(TemplateItem::Base(F)),
                I => match self.dna.pop_front().ok_or(EarlyFinish)? {
                    C => result.push(TemplateItem::Base(P)),
                    F | P => {
                        let l = self.nat()?;
                        result.push(TemplateItem::Ref(self.nat()?, l));
                    }
                    I => match self.dna.pop_front().ok_or(EarlyFinish)? {
                        C | F => break Ok(Template(result)),
                        P => result.push(TemplateItem::Length(self.nat()?)),
                        I => {
                            if self.dna.len() < 7 {
                                self.rna.extend(self.dna.clone());
                                self.dna.clear();
                                break Err(EarlyFinish);
                            }
                            for _ in 0..7 {
                                self.rna.push(self.dna.pop_front().unwrap());
                            }
                        }
                    },
                },
            }
        }
    }

    fn nat(&mut self) -> CanFinishEarly<u32> {
        let mut stack = vec![];
        loop {
            match self.dna.pop_front().ok_or(EarlyFinish)? {
                P => {
                    let mut result = 0;
                    for x in stack.into_iter().rev() {
                        result = result * 2 + x;
                    }
                    break Ok(result);
                }
                I | F => stack.push(0),
                C => stack.push(1),
            }
        }
    }

    fn as_nat(mut n: u32) -> Vec<Base> {
        let mut result = vec![];
        while n > 0 {
            if n % 2 == 0 {
                result.push(I);
            } else {
                result.push(C);
            }
            n /= 2;
        }
        result.push(P);
        result
    }

    fn consts(&mut self) -> Vec<Base> {
        let mut result = vec![];
        loop {
            if self.dna.is_empty() {
                break;
            }
            match self.dna[0] {
                C => {
                    self.dna.pop_front();
                    result.push(I);
                }
                F => {
                    self.dna.pop_front();
                    result.push(C);
                }
                P => {
                    self.dna.pop_front();
                    result.push(F);
                }
                I => {
                    if self.dna.is_empty() {
                        break;
                    }
                    match self.dna[1] {
                        C => {
                            self.dna.pop_front();
                            self.dna.pop_front();
                            result.push(P);
                        }
                        _ => break,
                    }
                }
            }
        }
        result
    }

    fn protect(l: u32, d: &Vec<Base>) -> Vec<Base> {
        let mut result = d.clone();
        for _ in 0..l {
            result = Self::quote(&result);
        }
        result
    }

    fn quote(d: &Vec<Base>) -> Vec<Base> {
        let mut result = vec![];
        for b in d {
            match b {
                I => result.push(C),
                C => result.push(F),
                F => result.push(P),
                P => result.extend(&[I, C]),
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
            ExecutionState::new(b"", b"CIIC").pattern(),
            Ok(Pattern(vec![Base(I)]))
        );
        assert_eq!(
            ExecutionState::new(b"", b"IIPIPICPIICICIIF").pattern(),
            Ok(Pattern(vec![GroupOpen, Skip(2), GroupClose, Base(P)])),
        );
        assert_eq!(
            ExecutionState::new(b"", b"IIPIPICPIICIFCCFPICIIF").pattern(),
            Ok(Pattern(vec![
                GroupOpen,
                Skip(2),
                GroupClose,
                Search(vec![I, C, F, P])
            ])),
        );
    }

    #[test]
    fn test_execute() {
        let mut state = ExecutionState::new(b"", b"IIPIPICPIICICIIFICCIFPPIICCFPC");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(state.dna, to_base_vec(b"PICFC"));
        state = ExecutionState::new(b"", b"IIPIPICPIICICIIFICCIFCCCPPIICCFPC");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(state.dna, to_base_vec(b"PIICCFCFFPC"));
        state = ExecutionState::new(b"", b"IIPIPIICPIICIICCIICFCFC");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(state.dna, to_base_vec(b"I"));
    }
}
