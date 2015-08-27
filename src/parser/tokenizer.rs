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
            if c == QUOTE {
                token.push(c);
                ready = true;
                self.stringing = !self.stringing;
            }
            else if self.stringing {
                token.push(c);
                if self.next_char == Some(QUOTE) {
                    ready = true;
                }
            }
            else if is_whitespace(c)  {

            }
            else if START_CHARS.contains(&c) || END_CHARS.contains(&c) {
                token.push(c);
                ready = true;
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

#[cfg(test)]
mod tests {
    use super::tokenize;
    use super::Token;

    fn token_vector(str: &'static str) -> Vec<Token> {
        tokenize(&String::from(str)).collect::<Vec<Token>>()
    }

    #[test]
    fn simple_forms() {
        assert_eq!(vec!("(","1","2","3",")"), token_vector("(1 2 3)"));
        assert_eq!(vec!("[","1","2","3","]"), token_vector("[1 2 3]"));
        assert_eq!(vec!("{","1","2","3","}"), token_vector("{1 2 3}"));
    }

    #[test]
    fn ignore_whitespace() {
        assert_eq!(vec!("(","1","2","3",")"), token_vector("(1,,, 2    3)"));
    }

    #[test]
    fn parse_strings() {
        assert_eq!(vec!("\"","multi word string { }","\""),
                   token_vector("\"multi word string { }\""));
        assert_eq!(vec!("\"","with line-breaks\n Second\r\nThird","\""),
                   token_vector("\"with line-breaks\n Second\r\nThird\""));
    }

    #[test]
    fn multi_forms() {
        assert_eq!(vec!("(","1",")","[","2","]","12","\"", ";;12", "\"", "{"),
                   token_vector("(1) [2] 12   \";;12\" {"));
        assert_eq!(vec!("\"", "str 1", "\"", "\"", "str 2", "\""),
                   token_vector("\"str 1\" \"str 2\" "));
    }

    #[test]
    fn dispatch() {
        assert_eq!(vec!("#(","1","2","3",")"), token_vector("#(1,,, 2    3)"));
        assert_eq!(vec!("#{","1","2","3","}"), token_vector("#{1 2 3}"));
        assert_eq!(vec!("#custom"), token_vector("#custom"));
        assert_eq!(vec!("#custom(", "1", "2", "3", ")"), token_vector("#custom(1 2 3)"));
        assert_eq!(vec!("#custom", "symbol"), token_vector("#custom symbol"));
    }
}
