use std::result::Result;
use std::time::Instant;

use crate::types::*;

struct ExecutionState {
    dna: DNA,
    rna: Vec<Base>,
    enable_debug_prints: bool,
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
    fn new(prefix: &[u8], dna_base: &[u8]) -> Self {
        ExecutionState {
            dna: DNA::new(&vec![to_base_vec(prefix), to_base_vec(dna_base)]),
            rna: vec![],
            enable_debug_prints: false,
        }
    }

    fn execute(&mut self) -> CanFinishEarly<()> {
        let mut i = 0;
        loop {
            if self.enable_debug_prints {
                println!("iteration {}", i);
                println!("dna length: {}", self.dna.len());
            }
            let time = Instant::now();
            let pattern = self.pattern()?;
            let template = self.template()?;
            if self.enable_debug_prints {
                println!("pattern: {}", pattern);
                println!("template: {}", template);
            }
            self.match_replace(pattern, template);
            if time.elapsed().as_millis() > 10 {
                println!("SLOW ITERATION {}: {}ms", i, time.elapsed().as_millis());
                self.dna.debug_print();
            }
            if self.enable_debug_prints {
                println!("rna length: {} ({})", self.rna.len() / 7, self.rna.len());
                println!();
            }
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
                        if i + s.len() > self.dna.len() {
                            return;
                        }
                        let mut window = self.dna.window(i, s.len());
                        loop {
                            if window.is_match(&s) {
                                i += window.offset() + s.len();
                                break;
                            }
                            if !window.next() {
                                return;
                            }
                        };
                    }
                }
                GroupOpen => {
                    c.push(i);
                }
                GroupClose => {
                    let start = c.pop().unwrap();
                    let end = i;
                    env.push(self.dna.slice(start..end));
                }
            }
        }
        if self.enable_debug_prints {
            println!("match length: {}", i);
        }
        let result = self.replace(template, &env);
        self.dna.truncate_front(i);
        self.dna.extend_front(result);
    }

    fn replace(&self, template: Template, env: &Vec<DNASlice>) -> Vec<DNAChunk> {
        if self.enable_debug_prints {
            for (i, p) in env.iter().enumerate() {
                print!("env[{}] = ", i);
                for b in self.dna.render(&p.slice(0..10.min(p.len()))) {
                    print!("{:?}", b);
                }
                println!("{} ({})", if p.len() > 10 { "..." } else { "" }, p.len());
            }
        }
        let mut result = vec![];
        let mut current_owned_chunk = vec![];
        for t in template.0 {
            use TemplateItem::*;
            match t {
                Base(b) => current_owned_chunk.push(b),
                Ref(n, l) => {
                    if !current_owned_chunk.is_empty() {
                        result.push(DNAChunk::Owned(current_owned_chunk));
                        current_owned_chunk = vec![];
                    }
                    if (n as usize) < env.len() {
                        if l == 0 {
                            result.push(DNAChunk::Slice(env[n as usize].clone()));
                        } else {
                            result.push(DNAChunk::Owned(Self::protect(
                                l,
                                self.dna.render(&env[n as usize]),
                            )));
                        }
                    }
                }
                Length(n) => {
                    if !current_owned_chunk.is_empty() {
                        result.push(DNAChunk::Owned(current_owned_chunk));
                        current_owned_chunk = vec![];
                    }
                    let length = if (n as usize) < env.len() {
                        env[n as usize].len()
                    } else {
                        0
                    };
                    result.push(DNAChunk::Owned(Self::as_nat(length as u32)));
                }
            }
        }
        if !current_owned_chunk.is_empty() {
            result.push(DNAChunk::Owned(current_owned_chunk));
        }
        result
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
                            self.rna.extend(self.dna.render(&self.dna.slice(0..7)));
                            self.dna.truncate_front(7);
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
                            self.rna.extend(self.dna.render(&self.dna.slice(0..7)));
                            self.dna.truncate_front(7);
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
                    while let Some(x) = stack.pop() {
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

    fn protect<'a>(l: u32, data: Vec<Base>) -> Vec<Base> {
        if l == 0 {
            panic!("This method should only be called if quouting is required");
        }
        let mut result = data;
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
        assert_eq!(
            state.dna.render(&state.dna.slice(0..state.dna.len())),
            to_base_vec(b"PICFC")
        );
        state = ExecutionState::new(b"", b"IIPIPICPIICICIIFICCIFCCCPPIICCFPC");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(
            state.dna.render(&state.dna.slice(0..state.dna.len())),
            to_base_vec(b"PIICCFCFFPC")
        );
        state = ExecutionState::new(b"", b"IIPIPIICPIICIICCIICFCFC");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(
            state.dna.render(&state.dna.slice(0..state.dna.len())),
            to_base_vec(b"I")
        );
        // Find "FF" and replace with nothing
        state = ExecutionState::new(b"", b"IFCPPIICIICPFF");
        let pattern = state.pattern().unwrap();
        let template = state.template().unwrap();
        state.match_replace(pattern, template);
        assert_eq!(state.dna.len(), 0);
    }
}
