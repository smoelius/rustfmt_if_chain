use proc_macro2::{LineColumn, Span};

#[derive(Debug)]
pub struct Rewriter<'original> {
    original: &'original str,
    rewritten: String,
    delta: isize,
    prev_end: Option<LineColumn>,
}

impl<'original> Rewriter<'original> {
    pub fn new(original: &'original str) -> Self {
        Self {
            original,
            rewritten: original.to_owned(),
            delta: 0,
            prev_end: None,
        }
    }

    pub fn contents(self) -> String {
        self.rewritten
    }

    pub fn rewrite(&mut self, span: Span, replacement: &str) {
        if let Some(end) = self.prev_end {
            assert!(
                end <= span.start(),
                "self = {:#?}, span.start() = {:?}, span.end() = {:?}",
                self,
                span.start(),
                span.end(),
            );
        };

        let (start, end) = self.offsets_from_span(span);

        let start = usize::try_from(start as isize + self.delta).unwrap();
        let end = usize::try_from(end as isize + self.delta).unwrap();

        let prefix = &self.rewritten.as_bytes()[..start];
        let suffix = &self.rewritten.as_bytes()[end..];

        self.rewritten = String::from_utf8(prefix.to_vec()).expect("`prefix` is not valid UTF-8")
            + replacement
            + &String::from_utf8(suffix.to_vec()).expect("`suffix` is not valid UTF-8");

        self.delta += replacement.as_bytes().len() as isize - end as isize + start as isize;

        self.prev_end = Some(span.end());
    }

    fn offsets_from_span(&self, span: Span) -> (usize, usize) {
        let (start, start_ascii) = self.offset_from_line_column(span.start());
        let (end, end_ascii) = self.offset_from_line_column(span.end());
        assert!(!end_ascii || start_ascii);
        // smoelius: `span`'s debug output doesn't seem to account for UTF-8.
        if end_ascii {
            assert_eq!(
                format!("{:?}", span),
                format!("bytes({}..{})", start + 1, end + 1),
                "self = {:#?}, span.start() = {:?}, span.end() = {:?}",
                self,
                span.start(),
                span.end(),
            );
        }
        (start, end)
    }

    fn offset_from_line_column(&self, line_column: LineColumn) -> (usize, bool) {
        let mut offset = 0;
        let mut ascii = true;
        let mut lines = self.original.split('\n');
        for _ in 1..line_column.line {
            let line = lines.next().unwrap();
            offset += line.as_bytes().len() + 1;
            ascii &= line.chars().all(|ch| ch.is_ascii());
        }
        let prefix = lines
            .next()
            .unwrap()
            .chars()
            .take(line_column.column)
            .collect::<String>();
        offset += prefix.as_bytes().len();
        ascii &= prefix.chars().all(|ch| ch.is_ascii());
        (offset, ascii)
    }
}
