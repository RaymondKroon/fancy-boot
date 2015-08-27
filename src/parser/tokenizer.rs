use std::str::Chars;
use super::{QUOTE,START_CHARS,END_CHARS,DISPATCH};

pub type Token = String;

pub struct TokenStream<'a> {
    rest: Chars<'a>,
    current_char: Option<char>,
    next_char: Option<char>,
    stringing: bool
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c == ','
}

impl<'a> TokenStream<'a> {
    fn read_next(&mut self) {
        self.current_char = self.next_char;
        self.next_char = self.rest.next();
    }
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let mut token = String::new();
        let mut ready = false;

        while let Some(c) = self.current_char {
            if self.stringing && self.next_char != Some(QUOTE) {
                token.push(c)
            }
            else if is_whitespace(c)  {
                if self.stringing {
                    token.push(c);
                }
            }
            else if END_CHARS.contains(&c) || (self.stringing && c == QUOTE) {
                token.push(c);
                ready = true;
                self.stringing = false;
            }
            else if START_CHARS.contains(&c) || c == QUOTE {
                token.push(c);
                ready = true;

                if c == QUOTE {
                    self.stringing = true;
                }
            }
            else if DISPATCH == c {
                token.push(c);
            }
            else {
                token.push(c);
                match self.next_char {
                    None => ready = true,
                    Some(n) =>
                        if is_whitespace(n) || END_CHARS.contains(&n) || n == QUOTE {
                            ready = true;
                        }
                }
            }

            self.read_next();
            if ready { break; }
        }

        if token.is_empty() {
            return None::<Token>;
        }
        else {
            return Some(token);
        }
    }
}

pub fn tokenize<'a>(str: &'a String) -> TokenStream<'a> {
    let mut rest = str.chars();
    let current = rest.next();
    let next = rest.next();
    TokenStream {rest: rest, current_char: current, next_char: next, stringing: false}
}
