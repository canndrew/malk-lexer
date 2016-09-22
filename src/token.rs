use tokens::TokensBuf;
use span::TextPos;

use std::borrow::Cow;

/// An element in a token tree.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'s> {
    /// A string of identifier characters.
    Ident(&'s str),
    
    /// A string of whitespace characters.
    Whitespace(&'s str),

    /// A valid symbol from the symbol table used when parsing.
    Symbol(&'s str),
    
    /// A bracket sequence of tokens.
    Bracket(char, TokensBuf<'s>),

    /// An unescaped string literal
    String(Cow<'s, str>),
}

/// A token with a position.
#[derive(Debug, Clone, PartialEq)]
pub struct Token<'s> {
    /// The position of the start of the token.
    pub start: TextPos,

    /// The kind of token.
    pub kind: TokenKind<'s>,
}

impl<'s> Token<'s> {
    /// Check whether a token is a whitespace token.
    pub fn is_whitespace(&self) -> bool {
        if let TokenKind::Whitespace(_) = self.kind {
            true
        }
        else {
            false
        }
    }
}

