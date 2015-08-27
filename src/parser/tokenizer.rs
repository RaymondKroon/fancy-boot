use super::{QUOTE,START_CHARS,END_CHARS,DISPATCH, COMMENT};

pub type Token = String;

pub trait Reader {
    fn current_char(&mut self) -> Option<char>;
    fn next_char(&mut self) -> Option<char>;
    fn pop(&mut self);
    fn flush_line(&mut self);
}

pub struct StringReader {
    chars: Vec<char>,
    size: usize,
    index: usize
}

impl StringReader {
    fn new (str: & String) -> StringReader {
        StringReader{chars: str.chars().collect(), size: str.len(), index: 0}
    }
}

impl Reader for StringReader {
    fn current_char(&mut self) -> Option<char> {
        if self.index < self.size {
            return Some(self.chars[self.index]);
        }
        else {
            return None::<char>;
        }
    }

    fn next_char(&mut self) -> Option<char> {
        if (self.index + 1) < self.size {
            return Some(self.chars[self.index + 1]);
        }
        else {
            return None::<char>;
        }
    }

    fn pop(&mut self) {
        self.index = self.index + 1;
    }

    fn flush_line(&mut self) {
        self.index = self.size + 1;
    }
}

pub struct TokenStream<T: Reader + Sized> {
    reader: T,
    stringing: bool
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c == ','
}

impl<T: Reader + Sized> Iterator for TokenStream<T> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        let mut token = String::new();
        let mut ready = false;

        while let Some(c) = self.reader.current_char() {
            if c == QUOTE {
                token.push(c);
                ready = true;
                self.stringing = !self.stringing;
            }
            else if self.stringing {
                token.push(c);
                if self.reader.next_char() == Some(QUOTE) {
                    ready = true;
                }
            }
            else if c == COMMENT {
                ready = true;
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
                match self.reader.next_char() {
                    None => ready = true,
                    Some(n) =>
                        if is_whitespace(n) || END_CHARS.contains(&n) || n == QUOTE {
                            ready = true;
                        }
                }
            }

            self.reader.pop();
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

pub fn tokenize<'a>(str: &'a String) -> TokenStream<StringReader> {
    let mut reader = StringReader::new(str);
    TokenStream {reader: reader, stringing: false}
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

    #[test]
    fn comment() {
        assert_eq!(vec!("[","[","]","]"), token_vector("[[]] ;; comment here"));
        assert_eq!(vec!("\"","with ;; in string too","\""),
                   token_vector("\"with ;; in string too\""));
    }
}
