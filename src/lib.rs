use std::io::Read;

use lexer::TokenKind;

mod lexer;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

// eat token
// ? peek first token, maybe just eat | use cursor or tokenize
// ? see second token
// AAA,AA,A,A
// B,B,B,B -> AAA,AA,A,A\nB,B,B,B
pub fn parse(string: &str) -> Vec<Vec<String>> {
    // vec![
    //     vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
    //     vec!["\"asb\"".to_owned(), "D".to_owned(), "E".to_owned()],
    // ]

    // cycle
    // or
    // cursor eating tokens -> push on matrix
    // (\n as command newline, ',' as new value)

    // "ab ""cd""" -> ab "cd"

    let mut lines = vec![];
    let mut line = vec![];
    let mut index: usize = 0;
    // summ all previous tokens length to find index of current token in string
    for token in lexer::tokenize(string) {
        // create len = 0
        // find TokenKind::Newline --> reset len to 0
        // AAA,AA,A,A\nB,B,B,B
        // &string[len..len+token_len]

        // see spaces and \n

        match token.kind {
            TokenKind::Comma
        }
         if token.kind == TokenKind::Newline {
            // \n
            lines.push(std::mem::take(&mut line));
        } else if token.kind != TokenKind::Comma {
            // !\n AND !,
            line.push(string[index..(index + token.len as usize)].to_owned());
        }
        // AAA,AA,A,Ai->\n<-B,B,B,B
        index += token.len as usize;
        // dbg!(&line);
    }
    lines.push(line);

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use tabled::settings::Style;

    fn assert_snapshot(string: &str, expected: &str) {
        let mut matrix = parse(string);

        for line in &mut matrix {
            for elem in line {
                // A -> " A "
                *elem = format!("{elem:?}");
            }
        }

        let mut table = tabled::builder::Builder::from_iter(matrix).build();

        table.with(Style::rounded());
        assert_eq!(table.to_string(), expected.trim());
    }

    #[test]
    fn smoke_table() {
        assert_snapshot("AAA,\"AA\",A#comment\n\"BBB\",BB,\"B\"", r#"
╭───────┬──────┬─────╮
│ "AAA" │ "AA" │ "A" │
├───────┼──────┼─────┤
│ "BBB" │ "BB" │ "B" │
╰───────┴──────┴─────╯"#)
    }

    #[test]
    fn empty_table() {
        assert_snapshot(",\n,", "")
    }

    #[test]
    fn row() {
        assert_snapshot("A,B,C,D", r#"
╭─────┬─────┬─────╮
│ "A" │ "B" │ "C" │
├─────┼─────┼─────┤"#)
    }

    #[test]
    fn column() {
        assert_snapshot("A\nB\nC\nD", r#"
╭─────╮
│ "A" │
├─────┤
│ "B" │
│ "C" │
│ "D" │
╰─────╯"#)
    }

    #[test]
    fn quoted() {
        assert_snapshot("\"AAA\",AA,\"A\"\nBBB,\"BB\",B", r#"
╭───────────┬──────────┬─────────╮
│ "\"AAA\"" │ "AA"     │ "\"A\"" │
├───────────┼──────────┼─────────┤
│ "BBB"     │ "\"BB\"" │ "B"     │
╰───────────┴──────────┴─────────╯"#)
    }

    // #[test]
    // fn comment() {
    //     assert_snapshot(, expected)
    // }
}
