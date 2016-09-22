use unicode_width::UnicodeWidthChar;

/// A position in a text document.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextPos {
    /// The column number from the start of the line. Some wierd characters may take up multiple
    /// columns or no columns.
    pub col: usize,
    /// The line number
    pub line: usize,
    /// The byte position from the start of the document.
    pub byte: usize,
}

/// A span of text in a text document.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    /// The start position of the span (inclusive).
    pub start: TextPos,
    /// The end position of the span (exclusive),
    pub end: TextPos,
}

impl TextPos {
    /// Create a TextPos at the start of a document.
    pub fn start() -> TextPos {
        TextPos {
            col: 0,
            line: 0,
            byte: 0,
        }
    }

    /// Advance a position by one character. Returns the character that was advanced past and the
    /// new position.
    pub fn next(self, src: &str) -> Option<(char, TextPos)> {
        let mut iter = src[self.byte..].char_indices();
        let (i, c) = match iter.next() {
            Some((_, c)) => {
                match iter.next() {
                    Some((i, _)) => (self.byte + i, c),
                    None => (src.len(), c),
                }
            },
            None => return None,
        };

        let pos = if c == '\n' {
            TextPos {
                col: 0,
                line: self.line + 1,
                byte: i,
            }
        }
        else {
            TextPos {
                col: self.col + c.width().unwrap_or(0),
                line: self.line,
                byte: i,
            }
        };
        Some((c, pos))
    }
}

