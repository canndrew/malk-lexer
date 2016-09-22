# malk-lexer

A unicode lexer for use as a first-pass when writing a parser.

The main function exported by this library is `lex` which takes a `&str` and a
table of valid symbols and converts them to a token tree.

The kinds of token recognized by the lexer are:
 * **Idents**: A string starting with a `XID_Start` character followed by a
   sequence of `XID_Continue` characters.
 * **Whitespace**: Any sequence of whitespace characters.
 * **Brackets**: Any bracket character, it's corresponding closing bracket and
   the tokens in-between returned as a sub-tree.
 * **Symbols**: Any string that appears in the symbol table provided to `lex`
 * **Strings**: A string enclosed with either `"` or `'` and which may contain
   escaped characters.

Patches welcome!

