use crate::offset_based_rewriter::{self, OffsetBasedRewriter};
use crate::offset_calculator::{self, OffsetCalculator};
use proc_macro2::{LineColumn, Span};
#[cfg(feature = "check-spans")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "check-spans")]
static BASE_NEXT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct Rewriter<'original> {
    line_column: LineColumn,
    offset_calculator: OffsetCalculator<'original>,
    offset_based_rewriter: OffsetBasedRewriter<'original>,
    #[cfg(feature = "check-spans")]
    base: usize,
}

impl<'original> Rewriter<'original> {
    pub fn new(original: &'original str) -> Self {
        #[cfg(feature = "check-spans")]
        let base = BASE_NEXT.fetch_add(1 + original.as_bytes().len(), Ordering::SeqCst);
        Self {
            line_column: LineColumn { line: 1, column: 0 },
            offset_calculator: OffsetCalculator::new(original),
            offset_based_rewriter: OffsetBasedRewriter::new(original),
            #[cfg(feature = "check-spans")]
            base,
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
        #[cfg(feature = "check-spans")]
        if end_ascii {
            let start = self.base + start;
            let end = self.base + end;
            assert_eq!(
                format!("{:?}", span),
                format!("bytes({}..{})", 1 + start, 1 + end),
                "self = {:#?}, span.start() = {:?}, span.end() = {:?}",
                self,
                span.start(),
                span.end(),
            );
        }
        (start, end)
    }
}
