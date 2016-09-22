use token::{Token, TokenKind};
use tokens::TokensBuf;
use span::TextPos;
use symbol_table::{InvalidSymbolTableError, validate_symbol_table};

use std::str::pattern::Pattern;
use std::borrow::Cow;
use std::char;
use unicode_brackets::UnicodeBrackets;

#[derive(Debug)]
pub enum LexError<'s> {
    InvalidSymbolTable(InvalidSymbolTableError<'s>),
    InvalidClosingBracket {
        open_pos: TextPos,
        close_pos: TextPos,
    },
    UnclosedBracket {
        open_pos: TextPos,
    },
    UnexpectedChar {
        pos: TextPos,
        c: char,
    },
    UnexpectedClosingBracket {
        pos: TextPos,
        c: char,
    },
    UnclosedString {
        start_pos: TextPos,
    },
    InvalidEscapeDigit {
        c: char,
        pos: TextPos,
    },
    InvalidEscapeCode {
        code: u32,
        pos: TextPos,
    },
    InvalidEscapeChar {
        c: char,
        pos: TextPos,
    },
}

/*
impl error::Error for LexError {
    fn cause(&self) -> Option<&error::Error> {
        match *self {
            InvalidSymbolTable(ref e) => Some(e),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        match *self {
            InvalidSymbolTable(_) => "Invalid symbol table",
            InvalidClosingBracket { .. } => "Invalid closing bracket",
            UnclosedBracket => "Unclosed bracket",
            UnexpectedChar => "Unexpected char",
            UnexpectedClosingBracket => "Unexpected closing bracket",
            UnclosedString => "Unclosed string",
            InvalidEscapeDigit => "Invalid escape digit",
            InvalidEscapeCode => "Invalid escape code",
            InvalidEscapeChar => "Invalid escape character",
        },
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InvalidSymbolTable(ref e) => {
                write!("Invalid symbol table: {}", e),
            },
            InvalidClosingBracket {
                open_pos: TextPos,
                close_pos: TextPos,
            } => {
                write!("Invalid closing bracket"),
            },
            UnclosedBracket {
                open_pos: TextPos,
            },
            UnexpectedChar {
                pos: TextPos,
                c: char,
            },
            UnexpectedClosingBracket {
                pos: TextPos,
                c: char,
            },
            UnclosedString {
                start_pos: TextPos,
            },
            InvalidEscapeDigit {
                c: char,
                pos: TextPos,
            },
            InvalidEscapeCode {
                code: u32,
                pos: TextPos,
            },
            InvalidEscapeChar {
                c: char,
                pos: TextPos,
            },
        }
    }
}
*/

/// The result of a succesful call to sub_lex
struct SubLex<'s> {
    /// The tokens succesfully lexed.
    tokens: TokensBuf<'s>,
    /// If lexing ended by hitting a closing bracket, the bracket and the position after it.
    terminator: Option<(char, TextPos)>,
}

fn sub_lex<'s>(start: TextPos, src: &'s str, symbols: &[&'s str]) -> Result<SubLex<'s>, LexError<'s>> {
    let mut tokens = Vec::new();
    let mut pos = start;
    'main: loop {
        let (c, p) = match pos.next(src) {
            Some(x) => x,
            None => return Ok(SubLex {
                tokens: TokensBuf {
                    tokens: tokens,
                    end: pos,
                },
                terminator: None,
            }),
        };
        if c.is_whitespace() {
            let mut end = p;
            loop {
                let (c, p) = match end.next(src) {
                    Some(x) => x,
                    None => break,
                };
                if !c.is_whitespace() {
                    break;
                }
                end = p;
            }

            let token = Token {
                kind: TokenKind::Whitespace(&src[pos.byte..end.byte]),
                start: pos,
            };

            tokens.push(token);
            pos = end;
            continue;
        }
        if c.is_xid_start() {
            let mut end = p;
            loop {
                let (c, p) = match end.next(src) {
                    Some(x) => x,
                    None => break,
                };
                if !c.is_xid_continue() {
                    break;
                }
                end = p;
            }

            let token = Token {
                kind: TokenKind::Ident(&src[pos.byte..end.byte]),
                start: pos,
            };

            tokens.push(token);
            pos = end;
            continue;
        }
        if c.is_open_bracket() {
            let sub = try!(sub_lex(p, src, symbols));
            match sub.terminator {
                Some((term_char, new_end)) => {
                    if term_char == c.to_close_bracket() {
                        let kind = TokenKind::Bracket(c, sub.tokens);
                        let token = Token {
                            kind: kind,
                            start: pos,
                        };
                        tokens.push(token);
                        pos = new_end;
                        continue;
                    }
                    else {
                        return Err(LexError::InvalidClosingBracket {
                            open_pos: pos,
                            close_pos: sub.tokens.end,
                        });
                    }
                },
                None => {
                    return Err(LexError::UnclosedBracket {
                        open_pos: pos,
                    });
                },
            }
        }
        if c.is_close_bracket() {
            return Ok(SubLex {
                tokens: TokensBuf {
                    tokens: tokens,
                    end: pos,
                },
                terminator: Some((c, p)),
            });
        }
        if c == '\'' || c == '"' {
            let token_start = pos;
            let next = |some_pos: TextPos| match some_pos.next(src) {
                Some(x) => Ok(x),
                None => Err(LexError::UnclosedString {
                    start_pos: token_start,
                }),
            };
            let from_hex = |some_char, its_pos| match some_char {
                '0'...'9' => Ok(some_char as u32 - '0' as u32),
                'a'...'f' => Ok(some_char as u32 - 'a' as u32 + 10),
                _ => Err(LexError::InvalidEscapeDigit {
                    c: some_char,
                    pos: its_pos,
                }),
            };
            let from_u32 = |some_u32, esc_pos| match char::from_u32(some_u32) {
                Some(c) => Ok(c),
                None => Err(LexError::InvalidEscapeCode {
                    code: some_u32,
                    pos: esc_pos,
                }),
            };
            let mut owned = None;
            let string_start = p;
            let mut p = p;
            loop {
                let (new_c, new_p) = try!(next(p));
                if new_c == c {
                    let cow = match owned {
                        Some(s) => Cow::Owned(s),
                        None => Cow::Borrowed(&src[string_start.byte..p.byte]),
                    };
                    let kind = TokenKind::String(cow);
                    let token = Token {
                        kind: kind,
                        start: pos,
                    };
                    tokens.push(token);
                    pos = new_p;
                    continue 'main;
                }
                if new_c == '\\' {
                    let (esc_c, esc_p) = try!(next(new_p));
                    let (unescaped, unescaped_end) = match esc_c {
                        '\'' => ('\'', esc_p),
                        '"'  => ('"',  esc_p),
                        '0'  => ('\0', esc_p),
                        't'  => ('\t', esc_p),
                        'n'  => ('\n', esc_p),
                        'r'  => ('\r', esc_p),
                        '\\' => ('\\', esc_p),
                        'x' => {
                            let (nib0, nib0_end) = try!(next(esc_p));
                            let (nib1, nib1_end) = try!(next(esc_p));
                            let nib0 = try!(from_hex(nib0, esc_p));
                            let nib1 = try!(from_hex(nib1, nib0_end));
                            (try!(from_u32((nib0 << 4) | nib1, p)), nib1_end)
                        },
                        _   => {
                            return Err(LexError::InvalidEscapeChar {
                                c: esc_c,
                                pos: new_p,
                            });
                        },
                    };
                    let took = owned.take();
                    let mut s = match took {
                        Some(s) => s,
                        None => String::from(&src[string_start.byte..p.byte]),
                    };
                    s.push(unescaped);
                    owned = Some(s);
                    p = unescaped_end;
                    continue;
                }
                p = new_p;
            }
        }

        let mut sym_end = p;
        loop {
            let sym_prefix = &src[pos.byte..sym_end.byte];
            let mut seen = false;
            let mut is_symbol = false;
            for this_symbol in symbols {
                if sym_prefix.is_prefix_of(this_symbol) {
                    match seen {
                        true => {
                            is_symbol = false;
                            break;
                        },
                        false => {
                            if this_symbol.len() == sym_prefix.len() {
                                is_symbol = true;
                            }
                            seen = true;
                        },
                    }
                }
            }

            if is_symbol {
                let token = Token {
                    kind: TokenKind::Symbol(sym_prefix),
                    start: pos,
                };
                tokens.push(token);
                pos = sym_end;
                continue 'main;
            };

            let new_p = match sym_end.next(src) {
                Some((c, new_p)) => {
                    if c.is_whitespace() ||
                       c.is_xid_start() ||
                       c.is_open_bracket() ||
                       c.is_close_bracket() ||
                       c == '\'' || c == '"' {
                        break;
                    }
                    new_p
                },
                _ => break,
            };
            sym_end = new_p;
        }
        return Err(LexError::UnexpectedChar {
            c: c,
            pos: pos,
        });
    }
}

pub fn lex<'s>(src: &'s str, symbols: &[&'s str]) -> Result<TokensBuf<'s>, LexError<'s>> {
    match validate_symbol_table(symbols) {
        Ok(()) => (),
        Err(e) => return Err(LexError::InvalidSymbolTable(e)),
    };

    let pos = TextPos::start();
    let sub = try!(sub_lex(pos, src, symbols));
    match sub.terminator {
        None => return Ok(sub.tokens),
        Some((c, dp)) => return Err(LexError::UnexpectedClosingBracket {
            pos: dp,
            c: c,
        }),
    };
}

