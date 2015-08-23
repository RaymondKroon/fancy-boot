//#![allow(unused_imports)]
//#![allow(unused_variables)]
//#![allow(dead_code)]

use std::str::Chars;

struct Tokenizer<'a> {
    rest: Chars<'a>,
    current_char: Option<char>,
    next_char: Option<char>
}

const START_CHARS: [char; 4] = ['(', '{', '[', '"'];
const END_CHARS: [char; 4] = [')', '}', ']', '"'];

const DISPATCH : char = '#';

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c == ','
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let mut token = String::new();
        let mut ready = false;

        while let Some(c) = self.current_char {
            if is_whitespace(c) {
            }
            else if END_CHARS.contains(&c) {
                token.push(c);
                ready = true;
            }
            else if START_CHARS.contains(&c) {
                token.push(c);
                ready = true;
            }
            else if DISPATCH == c {
                token.push(c)
            }
            else {
                token.push(c);
                match self.next_char {
                    None => break,
                    Some(n) =>
                        if is_whitespace(n) || END_CHARS.contains(&n) {
                            ready = true;
                        }
                }
            }

            self.current_char = self.next_char;
            self.next_char = self.rest.next();

            if ready { break; }
        }

        if token.is_empty() {
            return None::<String>;
        }
        else {
            return Some(token);
        }

    }
}

fn tokenize<'a>(str: &'a str) -> Tokenizer<'a> {
    let mut rest = str.chars();
    let current = rest.next();
    let next = rest.next();
    Tokenizer {rest: rest, current_char: current, next_char: next}
}

fn main() {
    let tokens: Tokenizer = tokenize("(test,,, 12234dd 2 3) { 1 2} [12, a] #{:a :b} [] #fancy[] (()) #\"regexp\" \"a\"");

    println!("{:?}", tokens.collect::<Vec<String>>());
}
