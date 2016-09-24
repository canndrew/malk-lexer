#![feature(pattern)]
#![feature(unicode)]

extern crate unicode_width;
extern crate unicode_xid;
extern crate unicode_brackets;

mod span;
mod token;
mod tokens;
mod symbol_table;
mod lex;

pub use span::{TextPos, Span};
pub use token::{TokenKind, Token};
pub use tokens::{TokensBuf, TokensRef};
pub use symbol_table::{InvalidSymbolTableError, validate_symbol_table};
pub use lex::{LexError, lex};

#[cfg(test)]
mod test {
    use token::{Token, TokenKind};
    use tokens::TokensBuf;
    use span::TextPos;
    use lex::{LexError, lex};

    use std::borrow::Cow;

    fn tp(p: usize) -> TextPos {
        TextPos {
            col: p,
            line: 0,
            byte: p,
        }
    }

    #[test]
    fn test() {
        let symbols = [
            "!@#",
            "$%^",
        ];
        let src = r#"()[{}] "wow\"\t\n\x23""floo"  !@#$%^hello_123"\u{394}""#;
        let tokens_buf = lex(src, &symbols).unwrap();
        assert_eq!(tokens_buf, TokensBuf {
            tokens: vec![
                Token {
                    kind: TokenKind::Bracket('(', TokensBuf {
                        tokens: vec![],
                        end: tp(1),
                    }),
                    start: tp(0),
                },
                Token {
                    kind: TokenKind::Bracket('[', TokensBuf {
                        tokens: vec![
                            Token {
                                kind: TokenKind::Bracket('{', TokensBuf {
                                    tokens: vec![],
                                    end: tp(4),
                                }),
                                start: tp(3),
                            },
                        ],
                        end: tp(5),
                    }),
                    start: tp(2),
                },
                Token {
                    kind: TokenKind::Whitespace(" "),
                    start: tp(6),
                },
                Token {
                    kind: TokenKind::String(Cow::Owned(String::from("wow\"\t\n#"))),
                    start: tp(7),
                },
                Token {
                    kind: TokenKind::String(Cow::Borrowed("floo")),
                    start: tp(22),
                },
                Token {
                    kind: TokenKind::Whitespace("  "),
                    start: tp(28),
                },
                Token {
                    kind: TokenKind::Symbol("!@#"),
                    start: tp(30),
                },
                Token {
                    kind: TokenKind::Symbol("$%^"),
                    start: tp(33),
                },
                Token {
                    kind: TokenKind::Ident("hello_123"),
                    start: tp(36),
                },
                Token {
                    kind: TokenKind::String(Cow::Owned(String::from("Δ"))),
                    start: tp(45),
                }
            ],
            end: tp(54),
        });
    }

    #[test]
    fn test_errors() {
        let src = "{]";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::InvalidClosingBracket {
            open_pos: tp(0),
            close_pos: tp(1),
        });

        let src = "[";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::UnclosedBracket {
            open_pos: tp(0),
        });

        let src = "#";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::UnexpectedChar {
            pos: tp(0),
            c: '#',
        });

        let src = "]";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::UnexpectedClosingBracket {
            pos: tp(0),
            c: ']',
        });

        let src = "\"";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::UnclosedString {
            start_pos: tp(0),
        });

        let src = r"'\x2g'";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::InvalidEscapeDigit {
            c: 'g',
            pos: tp(4),
        });

        let src = r"'\u{110000}'";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::InvalidEscapeCode {
            code: 0x110000,
            pos: tp(1),
        });

        let src = r"'\q'";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::InvalidEscapeChar {
            c: 'q',
            pos: tp(2),
        });

        let src = r"'\u{123456789}'";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::InvalidUnicodeEscape {
            pos: tp(1),
        });

        let src = r"'\u123'";
        let err = lex(src, &[]).unwrap_err();
        assert_eq!(err, LexError::InvalidUnicodeEscapeSyntax {
            pos: tp(3),
        });
    }
}

