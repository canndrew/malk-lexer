use span::{TextPos, Span};
use token::Token;

#[derive(Debug, Clone, PartialEq)]
pub struct TokensBuf<'s> {
    pub tokens: Vec<Token<'s>>,
    pub end: TextPos,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokensRef<'t, 's: 't> {
    pub tokens: &'t [Token<'s>],
    pub end: TextPos,
}

impl<'s> TokensBuf<'s> {
    /// Produce a `TokensRef` from this `TokenBuf`
    pub fn borrow<'t>(&'t self) -> TokensRef<'s, 't> {
        TokensRef {
            tokens: &self.tokens[..],
            end: self.end,
        }
    }
}

impl<'t, 's: 't> TokensRef<'t, 's> {
    /// Slice a range of tokens between `start_index` (inclusive) and `end_index` (exclusive).
    pub fn range(&self, start_index: usize, end_index: usize) -> TokensRef<'t, 's> {
        TokensRef {
            tokens: &self.tokens[start_index..end_index],
            end: match self.tokens.get(end_index + 1) {
                Some(token) => token.start,
                None        => self.end,
            },
        }
    }

    /// Slice a range of tokens between `start_index` and the end.
    pub fn range_from(&self, start_index: usize) -> TokensRef<'t, 's> {
        TokensRef {
            tokens: &self.tokens[start_index..],
            end: self.end,
        }
    }

    /// Split around the token at `index` returning all the tokens before it and all the tokens
    /// after it.
    pub fn split_around(&self, index: usize) -> (TokensRef<'t, 's>, TokensRef<'t, 's>) {
        let l = TokensRef {
            tokens: &self.tokens[..index],
            end: self.tokens[index].start,
        };
        let r = TokensRef {
            tokens: &self.tokens[(index + 1)..],
            end: self.end,
        };
        (l, r)
    }

    /// Trim whitespace tokens from both sides.
    pub fn trim_whitespace(&self) -> TokensRef<'t, 's> {
        let mut start_index = None;
        for (index, token) in self.tokens.iter().enumerate() {
            if !token.is_whitespace() {
                start_index = Some(index);
                break;
            }
        }
        let start_index = match start_index {
            Some(start_index) => start_index,
            None => return TokensRef {
                tokens: &[],
                end: self.end,
            },
        };

        let mut end_index = None;
        for (index, token) in self.tokens.iter().enumerate().rev() {
            if !token.is_whitespace() {
                end_index = Some(index);
                break;
            }
        }
        let end_index = end_index.unwrap() + 1;
        let end_pos = match end_index == self.tokens.len() {
            true  => self.end,
            false => self.tokens[end_index].start,
        };
        TokensRef {
            tokens: &self.tokens[start_index..end_index],
            end: end_pos,
        }
    }

    /// Get the span of this `TokensRef`
    pub fn span(&self) -> Span {
        Span {
            start: match self.tokens.first() {
                None => self.end,
                Some(t) => t.start,
            },
            end: self.end,
        }
    }
}


