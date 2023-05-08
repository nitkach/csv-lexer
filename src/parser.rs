use crate::lexer::{
    self,
    TokenKind::{self, *},
    Value,
};

// lib.rs > lexer
// lib.rs > module > parser

// crate > lib.rs

// super > module

pub(crate) struct ParsingContext {
    prev: Option<TokenKind>,
    line: Vec<String>,
    lines: Vec<Vec<String>>,
    index: usize,
}

impl ParsingContext {
    pub(crate) fn new() -> ParsingContext {
        ParsingContext {
            prev: None,
            line: vec![],
            lines: vec![],
            index: 0,
        }
    }

    pub(crate) fn parse(mut self, string: &str) -> Vec<Vec<String>> {
        for token in lexer::tokenize(string) {
            // create len = 0
            // find TokenKind::Newline --> reset len to 0
            // AAA,AA,A,\nB,B,B,B
            // &string[len..len+token_len]

            // see spaces and \n

            match (&self.prev, &token.kind) {
                (None | Some(Comma | Newline), Comma) => {
                    self.push_empty_value();
                }
                (None, Comment | Newline) => {}
                // A
                // .. ,

                // \n
                // ,

                // #comment ,
                (Some(Comment), Comma) => {
                    unreachable!()
                }
                // A, | "A",
                (Some(Value(_)), Comma) => {}

                // ,#comment
                (Some(Comma), Comment) => {
                    self.push_empty_value();
                }
                (Some(Comment), Comment) => {
                    unreachable!()
                }
                // \n
                // #comment
                // A#comment
                (Some(Newline | Value(_)), Comment) => {}

                // ,\n
                (Some(Comma), Newline) => {
                    self.push_empty_value();
                    self.push_line();
                }
                // #comment\n
                (
                    // some(comment) .is_empty
                    // \n #comment\n
                    // \n\n
                    // A\n | "A"\n
                    Some(Comment | Newline | Value(_)),
                    Newline,
                ) => {
                    self.push_line();
                }

                // ,A | ,"A"
                // \nA | \n"A"
                (None | Some(Comma | Newline), Value(value)) => {
                    let slice = &string[self.index..(self.index + token.len as usize)];
                    let to_push = match value {
                        Value::Quoted => lexer::unescape(slice),
                        Value::Bare => slice.to_owned(),
                    };
                    self.line.push(to_push);
                }
                // #commentA | #comment"A"
                (Some(Comment), Value(_)) => {
                    unreachable!()
                }
                // A"A" | "A"A
                (Some(Value(_)), Value(_)) => {
                    panic!()
                }
            }

            self.prev = Some(token.kind);

            // AAA,AA,A,Ai->\n<-B,B,B,B
            self.index += token.len as usize;
            // dbg!(&line);
        }
        // check: .is_empty
        // CHECK: line.is_empty
        // create function
        // parsing context: create struct with fields (bindings)
        // cursor store context
        // [Token1, Token2], cursor name:
        // cursor = tokenized_context
        // data structures and bags with context (data)
        // cursor describes process | token decribes type
        if let Some(Comma) = self.prev {
            self.push_empty_value();
        }

        self.push_line();
        self.lines
    }

    fn push_empty_value(&mut self) {
        self.line.push("".to_owned());
    }

    fn push_line(&mut self) {
        if self.line.is_empty() {
            return;
        }
        self.lines.push(std::mem::take(&mut self.line));
    }
}

#[cfg(test)]
mod tests {
    use tabled::settings::Style;

    #[track_caller]
    fn assert_snapshot(string: &str, expected: &str) {
        let mut matrix = crate::parse(string);

        for line in &mut matrix {
            for elem in line {
                // A -> " A "
                *elem = format!("{elem:?}");
            }
        }
        // dbg!(&matrix);
        let mut table = tabled::builder::Builder::from_iter(matrix);
        table.set_default_text("-");
        let mut table = table.build();

        table.with(Style::rounded());
        assert_eq!(table.to_string(), expected.trim());
    }

    #[test]
    fn smoke() {
        assert_snapshot(
            // "AAA" -> AAA
            // "AA""BB""A" -> AA"BB"A
            "AAA,\"AA\",A#comment\n\"BBB\",BB,\"B\"",
            r#"
╭───────┬──────┬─────╮
│ "AAA" │ "AA" │ "A" │
├───────┼──────┼─────┤
│ "BBB" │ "BB" │ "B" │
╰───────┴──────┴─────╯"#,
        )
    }

    #[test]
    fn empty() {
        assert_snapshot(
            ",\n,",
            r#"
╭────┬────╮
│ "" │ "" │
├────┼────┤
│ "" │ "" │
╰────┴────╯"#,
        )
    }

    #[test]
    fn row() {
        assert_snapshot(
            "A,B,C,D",
            r#"
╭─────┬─────┬─────┬─────╮
│ "A" │ "B" │ "C" │ "D" │
├─────┼─────┼─────┼─────┤"#,
        )
    }

    #[test]
    fn column() {
        assert_snapshot(
            "A\nB\nC\nD",
            r#"
╭─────╮
│ "A" │
├─────┤
│ "B" │
│ "C" │
│ "D" │
╰─────╯"#,
        )
    }

    #[test]
    fn quoted() {
        assert_snapshot(
            "\"AAA\",AA,\"A\"\nBBB,\"BB\",B",
            r#"
╭───────┬──────┬─────╮
│ "AAA" │ "AA" │ "A" │
├───────┼──────┼─────┤
│ "BBB" │ "BB" │ "B" │
╰───────┴──────┴─────╯"#,
        )
    }

    #[test]
    fn comment() {
        assert_snapshot(
            "A,B,C,#comment",
            r#"
╭─────┬─────┬─────┬────╮
│ "A" │ "B" │ "C" │ "" │
├─────┼─────┼─────┼────┤"#,
        )
    }

    #[test]
    fn comma() {
        assert_snapshot(
            "A,B,,C,D",
            r#"
╭─────┬─────┬────┬─────┬─────╮
│ "A" │ "B" │ "" │ "C" │ "D" │
├─────┼─────┼────┼─────┼─────┤"#,
        )
    }

    #[test]
    fn several_newlines() {
        assert_snapshot("A\n\n\n", r#"
╭─────╮
│ "A" │
├─────┤"#)
    }

    #[test]
    fn several_quoted() {
        assert_snapshot(r#""""""""#, r#"
╭────────╮
│ "\"\"" │
├────────┤"#)
    }
}
