
mod cursor;
mod unescape;

use cursor::Cursor;
pub(crate) use unescape::unescape;

#[derive(Debug)]
pub(crate) struct Token {
    pub(crate) len: u32,
    pub(crate) kind: TokenKind,
}

#[derive(Debug, PartialEq)]
pub(crate) enum TokenKind {
    /// `,`
    Comma,
    /// `#`
    Comment,
    /// `\n` or `\r\n`
    Newline,
    Value(Value),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Value {
    /// `"Foo \r""Baz"""`
    Quoted,
    /// `Foo Bar`
    Bare,
}

impl Token {
    fn new(kind: TokenKind, len: u32) -> Token {
        Token { len, kind }
    }
}

impl Cursor<'_> {
    /// Parses a token from the input string.
    pub fn eat_token(&mut self) -> Option<Token> {
        // struct Cursor { len_remaining, chars }
        //
        //    token_len_and_remaining = 9|remaining = chars.as_str().len() = 5
        //      /---|----\
        // Ford,Mare,1950
        //      |   ^
        //      |   chars.next() -> Some(',')
        //      |
        //      ^- (last reset position)
        //         token_len = token_len_and_remainging - chars.as_str().len() = 4
        //

        let first_char = self.eat_char()?;

        let token_kind = match first_char {
            '#' => self.eat_comment(),
            ',' => TokenKind::Comma,
            '\n' => TokenKind::Newline,
            '\r' if self.peek_first() == Some('\n') => {
                self.eat_char();
                TokenKind::Newline
            }
            '"' => self.eat_quoted(),
            _ => self.eat_bare(),
        };
        let res = Token::new(token_kind, self.token_len());
        self.reset_token_len();
        Some(res)
    }

    fn eat_comment(&mut self) -> TokenKind {
        while let Some(char) = self.peek_first() {
            if char == '\n' || (char == '\r' && self.peek_second() == Some('\n')) {
                break;
            }
            self.eat_char();
        }
        TokenKind::Comment
    }

    fn eat_quoted(&mut self) -> TokenKind {
        while let Some(char) = self.peek_first() {
            match (char, self.peek_second()) {
                ('"', Some('"')) => {
                    self.eat_char();
                    self.eat_char();
                }
                ('"', _) => {
                    self.eat_char();
                    break;
                }
                _ => {
                    self.eat_char();
                }
            }
        }
        TokenKind::Value(Value::Quoted)
    }

    fn eat_bare(&mut self) -> TokenKind {
        while let Some(char) = self.peek_first() {
            let flag = matches!(char, ',' | '#' | '\n');
            if flag || (char == '\r' && self.peek_second() == Some('\n')) {
                break;
            }
            self.eat_char();
        }
        TokenKind::Value(Value::Bare)
    }
}

// Box<[Token]>
pub(crate) fn tokenize(string: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut cursor = Cursor::new(string);

    while let Some(token) = cursor.eat_token() {
        tokens.push(token);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(string: &str) -> String {
        let tokens = tokenize(string);

        let mut snapshot = vec![];

        for elem in tokens {
            let Token { len, kind } = elem;
            snapshot.push(format!("{len}:{kind:?}"));
        }

        snapshot.join(",")
    }

    #[test]
    fn smoke() {
        assert_eq!(
            make_snapshot("A,B,C,D#comment"),
            "\
            1:Value(Bare),\
            1:Comma,\
            1:Value(Bare),\
            1:Comma,\
            1:Value(Bare),\
            1:Comma,\
            1:Value(Bare),\
            8:Comment"
        )
    }

    #[test]
    fn comma() {
        assert_snapshot(",", "1:Comma");
        assert_snapshot(",,", "1:Comma,1:Comma");
        assert_snapshot(",#comment", "1:Comma,8:Comment");
        assert_snapshot(",\n", "1:Comma,1:Newline");
        assert_snapshot(",A", "1:Comma,1:Value(Bare)");
        assert_snapshot(",\"A\"", "1:Comma,3:Value(Quoted)");

        assert_snapshot(
            "\r,\n,\",\r\"",
            "1:Value(Bare),1:Comma,1:Newline,1:Comma,4:Value(Quoted)",
        )
    }

    #[test]
    fn newline() {
        assert_snapshot("\n", "1:Newline");
        assert_snapshot("\n\n", "1:Newline,1:Newline");
        assert_snapshot("\n#comment", "1:Newline,8:Comment");
        assert_snapshot("\n,", "1:Newline,1:Comma");
        assert_snapshot("\nA", "1:Newline,1:Value(Bare)");
        assert_snapshot("\n\"A\"", "1:Newline,3:Value(Quoted)");
        assert_snapshot("\r\n", "2:Newline");

        assert_snapshot("\n\r", "1:Newline,1:Value(Bare)");
        assert_snapshot("\r\r\n", "1:Value(Bare),2:Newline");
    }

    #[test]
    fn comment() {
        assert_snapshot("#", "1:Comment");
        assert_snapshot("##", "2:Comment");
        assert_snapshot("#\n", "1:Comment,1:Newline");
        assert_snapshot("#,", "2:Comment");
        assert_snapshot("#A", "2:Comment");
        assert_snapshot("#\"A\"", "4:Comment");
    }

    #[test]
    fn value_bare() {
        assert_snapshot("A", "1:Value(Bare)");
        assert_snapshot("A#", "1:Value(Bare),1:Comment");
        assert_snapshot("A\n", "1:Value(Bare),1:Newline");
        assert_snapshot("A,", "1:Value(Bare),1:Comma");
        assert_snapshot("AA", "2:Value(Bare)");
        assert_snapshot("A\"A\"", "4:Value(Bare)");

        assert_snapshot("П", "2:Value(Bare)");

        assert_snapshot("Q🔥,", "5:Value(Bare),1:Comma");

        assert_snapshot("A\r123", "5:Value(Bare)");
        assert_snapshot("\r123", "4:Value(Bare)");

        assert_snapshot("\r\r", "2:Value(Bare)");
    }

    #[test]
    fn value_quoted() {
        assert_snapshot("\"A\"", "3:Value(Quoted)");
        assert_snapshot("\"A\"#", "3:Value(Quoted),1:Comment");
        assert_snapshot("\"A\"\n", "3:Value(Quoted),1:Newline");
        assert_snapshot("\"A\",", "3:Value(Quoted),1:Comma");
        assert_snapshot("\"A\"A", "3:Value(Quoted),1:Value(Bare)");
        assert_snapshot("\"A\"\"A\"", "6:Value(Quoted)");

        assert_snapshot("\"П\"", "4:Value(Quoted)");

        assert_snapshot("\"Q🔥\",", "7:Value(Quoted),1:Comma");

        assert_snapshot(&"\"".repeat(6), "6:Value(Quoted)");

        assert_snapshot("\"\n,\r\n#comment\n\r\n\"", "17:Value(Quoted)");
    }

    #[track_caller]
    fn assert_snapshot(snapshot: &str, string: &str) {
        assert_eq!(make_snapshot(snapshot), string);
    }
    // Value(Value),
    // test common and special cases
}
