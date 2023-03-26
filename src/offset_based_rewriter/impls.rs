#![cfg_attr(nightly, cast_checks::enable)]

use super::Interface;

#[derive(Debug)]
pub struct LazyRewriter<'original> {
    original: &'original str,
    rewritten: String,
    offset: usize,
}

#[derive(Debug)]
pub struct EagerRewriter {
    rewritten: String,
    delta: isize,
}

impl<'original> LazyRewriter<'original> {
    pub fn new(original: &'original str) -> Self {
        Self {
            original,
            rewritten: String::new(),
            offset: 0,
        }
    }
}

impl EagerRewriter {
    #[allow(dead_code)]
    pub fn new(original: &str) -> Self {
        Self {
            rewritten: original.to_owned(),
            delta: 0,
        }
    }
}

impl<'original> Interface for LazyRewriter<'original> {
    fn contents(mut self) -> String {
        self.rewritten += &self.original[self.offset..];

        self.rewritten
    }

    fn rewrite(&mut self, start: usize, end: usize, replacement: &str) {
        assert!(self.offset <= start);

        self.rewritten += &self.original[self.offset..start];
        self.rewritten += replacement;

        self.offset = end;
    }
}

impl Interface for EagerRewriter {
    fn contents(self) -> String {
        self.rewritten
    }

    #[allow(clippy::cast_possible_wrap)]
    fn rewrite(&mut self, start: usize, end: usize, replacement: &str) {
        let start = usize::try_from(start as isize + self.delta).unwrap();
        let end = usize::try_from(end as isize + self.delta).unwrap();

        let prefix = &self.rewritten.as_bytes()[..start];
        let suffix = &self.rewritten.as_bytes()[end..];

        self.rewritten = String::from_utf8(prefix.to_vec()).expect("`prefix` is not valid UTF-8")
            + replacement
            + &String::from_utf8(suffix.to_vec()).expect("`suffix` is not valid UTF-8");

        self.delta += replacement.as_bytes().len() as isize - end as isize + start as isize;
    }
}
