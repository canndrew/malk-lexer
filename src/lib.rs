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
    use lex::lex;

    use std::borrow::Cow;

    #[test]
    fn test() {
        let tp = |p| TextPos {
            col: p,
            line: 0,
            byte: p,
        };
        let symbols = [
            "!@#",
            "$%^",
        ];
        let src = r#"()[{}] "wow\"\t\n\x22""floo"  !@#$%^hello_123"#;
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
                    kind: TokenKind::String(Cow::Owned(String::from("wow\"\t\n\""))),
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
            ],
            end: tp(45),
        });
    }
}

