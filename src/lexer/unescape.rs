struct UnescapeContext {
    state: State,
    acc: String,
}

#[derive(Debug, PartialEq)]
enum State {
    Start,
    Middle,
    MetQuote,
}

impl UnescapeContext {
    fn eat_char(&mut self, char: char) {
        match (&self.state, char) {
            (State::Start, '"') => {
                self.state = State::Middle;
            }
            (State::Start, _) => {
                unreachable!();
            }
            (State::Middle, '"') => {
                self.state = State::MetQuote;
            }
            (State::Middle, _) => {
                self.acc.push(char);
            }
            // "AA""B""A" -> AA"B"A
            // "AA""B"""
            (State::MetQuote, '"') => {
                self.acc.push(char);
                self.state = State::Middle;
            }
            (State::MetQuote, _) => {
                unreachable!();
            }
        }
    }
}

pub(crate) fn unescape(string: &str) -> String {
    let mut context = UnescapeContext {
        state: State::Start,
        acc: "".to_owned(),
    };

    // "AA""B""A" -> AA"B"A
    // string.trim_matches('"').replace("\"\"", "\"");

    for char in string.chars() {
        context.eat_char(char);
    }

    assert_eq!(context.state, State::MetQuote);

    context.acc
}
