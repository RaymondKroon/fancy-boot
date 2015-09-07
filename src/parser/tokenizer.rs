use std::error::Error as Err;
use std::fs::File;
use std::io::{BufReader, BufRead, Read};
use std::path::Path;

const BANG: char = '!';
const COMMENT: char = ';';
const DEREF: char = '@';
const DISPATCH : char = '#';
const DOUBLE_QUOTE: char = '"';
const LIST_START: char = '(';
const LIST_END: char = ')';
const MAP_START: char = '{';
const MAP_END: char = '}';
const SINGLE_QUOTE: char = '\'';
const SYNTAX_QUOTE: char = '`';
const TILDE: char = '~';
const VECTOR_START: char = '[';
const VECTOR_END: char = ']';

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Discard,
    Dispatch,
    Deref,
    FunctionStart,
    ListStart,
    ListEnd,
    Literal(String),
    MapStart,
    MapEnd,
    Quote,
    Regex(String),
    SetStart,
    String(String),
    SyntaxQuote,
    Unquote,
    UnquoteSplicing,
    VectorStart,
    VectorEnd
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenInfo {
    line: usize,
    char: usize
}

impl TokenInfo {
    fn new(pos: (usize, usize)) -> TokenInfo {
        TokenInfo{line: pos.0, char: pos.1}
    }
}

pub struct TokenStream<T: Scanner + Sized> {
    scanner: T,
    stringing: bool,
    regex: bool,
    waiting: Option<(Token, TokenInfo)>
}

impl TokenStream {
    fn new<T>(scanner: T) -> TokenStream<T> {
        TokenStream {scanner: scanner, stringing: false, regex: false,
        waiting: None::<Token>}
    }

    fn is_next(req: char) -> bool {
        if let Some(c) = self.scanner.next_char() {
            return c == req;
        }

        return false;
    }

    fn is_prev_whitespace() -> bool {
        if let Some(c) = self.scanner.previous_char() {
            return is_whitespace(c);
        }

        return false;
    }
}

impl<T: Scanner + Sized> Iterator for TokenStream<T> {
    type Item = (Token, TokenInfo);

    fn next(&mut self) -> Option<(Token, TokenInfo)> {

        if let Some(waiting) = self.waiting {
            self.waiting = None::<(Token, TokenInfo)>;
            return waiting;
        }

        let mut buffer = String::new();
        let token_info = TokenInfo::new(self.reader.position());
        let mut token = None::<(Token, TokenInfo)>;
        let mut ready = false;

        fn ready() {
            ready = true;
        }

        fn ret(t: Token) {

            if !buffer.is_empty() {
                token = Some(Token::Literal(buffer), token_info);
                self.waiting = Some(t, TokenInfo::new(self.reader.position()));
            }
            else {
                token = Some(t, token_info);
            }

            ready();
        }

        fn flush_line() {
            self.reader.flush_line();
            if !buffer.is_empty() {
                token = Token::literal(buffer);
                ready = true;
            }
        }

        while let Some(c) = self.reader.current_char() {

            match c {
                DOUBLE_QUOTE if self.stringing => {
                    self.stringing = false;

                    if self.regex {
                        ret(Token::Regex(buffer));
                        self.regex = false;
                    }
                    else {
                        ret(Token::String(buffer));
                    }
                },
                DOUBLE_QUOTE if !self.stringing => {
                    self.stringing = true;
                },
                _ if self.stringing => {
                    buffer.push(c);
                },
                COMMENT => self.scanner.flush_line(),
                _ if is_whitespace(c) => {},
                LIST_START => ret(Token::ListStart),
                LIST_END => ret(Token::ListEnd),
                VECTOR_START => ret(Token::VectorStart),
                VECTOR_END => ret(Token::VectorEnd),
                MAP_START => ret(Token::MapStart),
                MAP_END => ret(Token::MapStart),
                DISPATCH if is_next(BANG) && is_prev_whitespace() => self.scanner.flush_line(),
                DISPATCH if is_next(LIST_START) => ret(Token::FunctionStart),
                DISPATCH if is_next(MAP_START) => ret(Token::set_start()),
                DISPATCH if is_next(DOUBLE_QUOTE) => {
                    self.scanner.pop();
                    self.regex = true;
                },
                SINGLE_QUOTE => ret(Token::Quote),
                SYNTAX_QUOTE => ret(Token::SyntaxQuote),
                TILDE if is_next(DEREF) => {
                    self.scanner.pop();
                    ret(Token::UnquoteSplicing);
                },
                TILDE => ret(Token::Unquote),
                _ => {
                    buffer.push(c);

                    if let Some(n) = self.reader.next_char() {
                        if is_whitespace(n) {
                            ret(Token::Literal(buffer))
                        }
                    }
                    else {
                        ret(Token::Literal(buffer));
                    }
                }
            }

            self.scanner.pop();
            if ready { break; }
        }

        return token;
    }
}

pub trait Scanner {
    fn current_char(&self) -> Option<char>;
    fn next_char(&self) -> Option<char>;
    fn previous_char(&self) -> Option<char>;
    fn pop(&mut self);
    fn flush_line(&mut self);
    fn position(&self) -> (usize, usize);
}

pub struct StringScanner {
    chars: Vec<char>,
    size: usize,
    index: usize
}

impl StringScanner {
    fn new (str: & String) -> StringScanner {
        StringScanner{chars: str.chars().collect(), size: str.len(), index: 0}
    }
}

impl Scanner for StringScanner {
    fn current_char(&self) -> Option<char> {
        if self.index < self.size {
            return Some(self.chars[self.index]);
        }
        else {
            return None::<char>;
        }
    }

    fn next_char(&self) -> Option<char> {
        if (self.index + 1) < self.size {
            return Some(self.chars[self.index + 1]);
        }
        else {
            return None::<char>;
        }
    }

    fn previous_char(&self) -> Option<char> {
        if (self.index - 1) > 0 {
            return Some(self.chars[self.index - 1]);
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

    fn position(&self) -> (usize, usize) {
        (0, index + 1)
    }
}

#[derive(Debug)]
pub struct LineScanner
{
    lines: Vec<(usize, Vec<char>)>,
    current: (usize, usize),
    next: (usize, usize),
    previous: (usize, usize)
}

impl LineScanner
{
    fn from_file (strpath: &String) -> Self {
        let path = Path::new(strpath);
        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}",
                           path.display(),
                           Err::description(&why)),
            Ok(file) => file,
        };

        let reader = BufReader::new(file);

        Self::from_buffer(reader)
    }

    fn from_buffer<R: BufRead>(reader: R) -> Self {
        let lines = reader.lines()
            .filter_map(|result| result.ok())
            .map(|s| (s.len(), s.chars().collect()))
            .collect::<Vec<(usize, Vec<char>)>>();

        LineScanner{lines: lines,
                    current: (0, 0),
                    next: (0, 1),
                    previous: (0, -1)
        }
    }

    fn read_char(&self, idx: (usize, usize)) -> Option<char> {
        let (l,c) = idx;
        if l < self.lines.len()
        {

            if c < 0 {
                return None::<char>;
            }
            else if c < self.lines[l].0 {
                return Some(self.lines[l].1[c]);
            }
            else { // insert \n as last char of line
                return Some('\n');
            }

        }
        else {
            return None::<char>;
        }
    }
}

impl Scanner for LineScanner {
    fn current_char(&self) -> Option<char> {
        self.read_char(self.current)
    }

    fn next_char(&self) -> Option<char> {
        self.read_char(self.next)
    }

    fn previous_char(&self) -> Option<char> {
        self.read_char(self.previous);
    }

    fn pop(&mut self) {

        let (cl,cc) = self.current;
        self.previous = (cl, cc);

        if cl < self.lines.len() {
            if cc >= self.lines[cl].0 {
                self.current = (cl + 1, 0);
            }
            else {
                self.current = (cl, cc + 1);
            }
        }

        let (nl, nc) = self.next;

        if nl < self.lines.len() {
            if nc >= self.lines[nl].0 {
                self.next = (nl + 1, 0);
            }
            else {
                self.next = (nl, nc + 1);
            }
        }
    }

    fn flush_line(&mut self) {
        self.previous = (self.current.0, self.current.1);
        self.current = (self.current.0 + 1, 0);
        self.next = (self.next.0 + 1, 1);
    }

    fn position(&self) -> (usize, usize) {
        let (l, c) = self.current;
        (l + 1, c + 1)
    }
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c == ','
}

pub fn tokenize(str: String) -> TokenStream<StringScanner> {
    let scanner = StringScanner::new(&str);
    TokenStream::new(scanner)
}

pub fn tokenize_file(path: String) -> TokenStream<LineScanner> {
    let scanner = LineScanner::from_file(&path);
    TokenStream::new(scanner)
}

pub fn tokenize_stream<R: BufRead>(buf_reader: R) -> TokenStream<LineScanner> {
    let scanner = LineScanner::from_buffer(buf_reader);
    TokenStream::new(scanner)
}

#[cfg(test)]
mod tests {
    use super::tokenize;
    use super::tokenize_file;
    use super::Token;

    fn token_vector(str: &'static str) -> Vec<Token> {
        tokenize(String::from(str)).collect::<Vec<Token>>()
    }

    fn tokens_from_file(path: &'static str) -> Vec<Token> {
        tokenize_file(String::from(path)).collect::<Vec<Token>>()
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
        assert_eq!(vec!("#custom", "(", "1", "2", "3", ")"), token_vector("#custom(1 2 3)"));
        assert_eq!(vec!("#custom", "symbol"), token_vector("#custom symbol"));
        assert_eq!(vec!("(", "symbol", "{"), token_vector("(symbol{"));
        assert_eq!(vec!("symbol", "("), token_vector("symbol("));
        assert_eq!(vec!("symbol", "["), token_vector("symbol["));
        assert_eq!(vec!("#symbol", "{"), token_vector("#symbol{"));
        assert_eq!(vec!("#{", "a"), token_vector("#{a"));
        assert_eq!(vec!("#(", "a"), token_vector("#(a"));
        assert_eq!(vec!("#", "[", "a"), token_vector("#[a"));
    }

    #[test]
    fn comment() {
        assert_eq!(vec!("[","[","]","]"), token_vector("[[]] ;; comment here"));
        assert_eq!(vec!("\"","with ;; in string too","\""),
                   token_vector("\"with ;; in string too\""));
    }

    #[test]
    fn read_file() {
        assert_eq!(vec!("(","test","1","2","3",")"),
                   tokens_from_file("resources/tokenizer/simple.fc"));

        assert_eq!(vec!("(","test","1","2","3",")"),
                   tokens_from_file("resources/tokenizer/multiline.fc"));

        assert_eq!(vec!("(", "defn","test","[","a","]","(","+","1","a",")",")"),
                   tokens_from_file("resources/tokenizer/withcomments.fc"));
    }
}
