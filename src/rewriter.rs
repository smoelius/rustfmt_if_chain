use crate::offset_based_rewriter::{self, OffsetBasedRewriter};
use crate::offset_calculator::{self, OffsetCalculator};
use proc_macro2::{LineColumn, Span};

#[derive(Debug)]
pub struct Rewriter<'original> {
    line_column: LineColumn,
    offset_calculator: OffsetCalculator<'original>,
    offset_based_rewriter: OffsetBasedRewriter<'original>,
}

impl<'original> Rewriter<'original> {
    pub fn new(original: &'original str) -> Self {
        Self {
            line_column: LineColumn { line: 1, column: 0 },
            offset_calculator: OffsetCalculator::new(original),
            offset_based_rewriter: OffsetBasedRewriter::new(original),
        }
    }

    pub fn contents(self) -> String {
        use offset_based_rewriter::Interface;

        self.offset_based_rewriter.contents()
    }

    pub fn rewrite(&mut self, span: Span, replacement: &str) {
        use offset_based_rewriter::Interface;

        assert!(
            self.line_column <= span.start(),
            "self = {:#?}, span.start() = {:?}, span.end() = {:?}",
            self,
            span.start(),
            span.end(),
        );

        let (start, end) = self.offsets_from_span(span);

        self.offset_based_rewriter.rewrite(start, end, replacement);

        self.line_column = span.end();
    }

    fn offsets_from_span(&mut self, span: Span) -> (usize, usize) {
        use offset_calculator::Interface;

        let (start, start_ascii) = self.offset_calculator.offset_from_line_column(span.start());
        let (end, end_ascii) = self.offset_calculator.offset_from_line_column(span.end());
        assert!(!end_ascii || start_ascii);
        // smoelius: `Span`'s debug output doesn't seem to account for UTF-8.
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
}
