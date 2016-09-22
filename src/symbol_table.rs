use std::str::pattern::Pattern;

/// Error returned by `validate_symbol_table`. Represents two sequences of symbols that cannot be
/// parsed unambiguously because they would appear as the same string of text.
#[derive(Debug)]
pub struct InvalidSymbolTableError<'s> {
    pub first: Vec<&'s str>,
    pub second: Vec<&'s str>,
}

/// Check that a symbol table can be used to parse unambiguously.
pub fn validate_symbol_table<'s>(symbols: &[&'s str]) -> Result<(), InvalidSymbolTableError<'s>> {
    fn recurse<'s>(symbols: &[&'s str],
                   complete_list: &mut Vec<&'s str>,
                   postfix_list: &mut Vec<&'s str>,
                   postfix: &'s str) -> Result<(), InvalidSymbolTableError<'s>>
    {
        for symbol in symbols.iter() {
            if postfix.is_prefix_of(symbol) {
                complete_list.push(*symbol);
                let new_postfix = &symbol[postfix.len()..];
                if new_postfix == "" {
                    return Err(InvalidSymbolTableError {
                        first: complete_list.clone(),
                        second: postfix_list.clone(),
                    })
                }
                try!(recurse(symbols, postfix_list, complete_list, new_postfix));
                let _ = complete_list.pop();
            }
        }
        Ok(())
    }

    for (i0, a) in symbols.iter().enumerate() {
        for (i1, b) in symbols.iter().enumerate() {
            if i0 == i1 {
                continue;
            }
            if a.is_prefix_of(b) {
                let mut complete_list = vec![*a];
                let mut postfix_list = vec![*b];
                let postfix = &b[a.len()..];
                try!(recurse(symbols,
                             &mut complete_list,
                             &mut postfix_list,
                             postfix));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std;

    #[test]
    fn test_invalid_symbol_table() {
        let symbols = [
            "!@",
            "#$",
            "%^",
            "!@#",
            "$%^",
        ];
        let err = validate_symbol_table(&symbols[..]).unwrap_err();
        let mut first = err.first;
        let mut second = err.second;
        if second.len() < first.len() {
            std::mem::swap(&mut first, &mut second);
        }
        assert_eq!(&first[..], ["!@#", "$%^"]);
        assert_eq!(&second[..], ["!@", "#$", "%^"]);
    }
}

