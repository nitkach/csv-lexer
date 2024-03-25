mod lexer;
mod parser;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

// eat token
// ? peek first token, maybe just eat | use cursor or tokenize
// ? see second token
// AAA,AA,A,A
// B,B,B,B -> AAA,AA,A,A\nB,B,B,B
pub fn parse(string: &str) -> Vec<Vec<String>> {
    let parser = parser::ParsingContext::new();
    parser.parse(string)
}
