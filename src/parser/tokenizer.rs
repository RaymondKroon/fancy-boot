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
const UNQUOTE: char = '~';
const VECTOR_START: char = '[';
const VECTOR_END: char = ']';

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    Discard(Discard),
    Dispatch(String),
    Deref(Deref),
    FunctionStart(FunctionStart),
    ListStart(ListStart),
    ListEnd(ListEnd),
    Literal(String),
    MapStart(MapStart),
    Quote(Quote),
    Regex(String),
    SetStart(SetStart),
    SetEnd(SetEnd),
    String(String),
    Unquote(Unquote),
    UnquoteSplicing(UnquoteSplicing),
    VectorStart(VectorStart),
    VectorEnd(VectorEnd)
}

impl Token {

    fn discard() -> Token { Token::Discard(Discard) }
    fn dispatch() -> Token { Token::Dispatch(Dispatch) }
    fn deref() -> Token { Token::Deref(Deref) }
    fn function_start() -> Token {Token::FunctionStart(FunctionStart)}
    fn list_start() -> Token { Token::ListStart(ListStart) }
    fn list_end() -> Token { Token::ListEnd(ListEnd) }
    fn literal(str: String) -> Token { Token::Literal(str) }
    fn map_start() -> Token { Token::MapStart(MapStart) }
    fn map_end() -> Token { Token::MapEnd(MapEnd) }
    fn quote() -> Token { Token::Quote(Quote) }
    fn regex(str: String) -> Token { Token::Regex(str) }
    fn set_start() -> Token { Token::SetStart(SetStart) }
    fn string(str: String) -> Token { Token::String(str) }
    fn unquote() -> Token { Token::Unquote(Unquote) }
    fn unquote_splicing() -> Token { Token::UnquoteSplicing(UnquoteSplicing) }
    fn vector_start() -> Token { Token::VectorStart(VectorStart) }
    fn vector_end() -> Token { Token::VectorEnd(VectorEnd) }

}

#[derive(Clone, Debug, Eq, PartialEq)] pub struct Discard;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct FunctionStart;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct ListStart;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct ListEnd;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct MapStart;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct MapEnd;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct Quote;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct Unquote;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct UnquoteSplicing;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct VectorStart;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct VectorEnd;
#[derive(Clone, Debug, Eq, PartialEq)] pub struct SetStart;

pub struct TokenInfo {
    line: usize,
    char: usize
}

impl TokenInfo {
    fn new(pos: (usize, usize)) -> TokenInfo {
        TokenInfo{line: pos.0, char: pos.1}
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

pub struct TokenStream<T: Scanner + Sized> {
    reader: T,
    stringing: bool
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() || c == ','
}

impl<T: Scanner + Sized> Iterator for TokenStream<T> {
    type Item = (Token, TokenInfo);

    fn next(&mut self) -> Option<(Token, TokenInfo)> {
        let mut buffer = String::new();
        let token_info = TokenInfo::new(self.reader.position());
        let mut token = None::<(Token, TokenInfo)>;
        let mut ready = false;


        fn is_next(req: char) -> bool {
            if let Some(c) = self.reader.next_char() {
                return c == req;
            }

            return false;
        }

        fn is_prev_whitespace() -> bool {
            if let Some(c) = self.reader.previous_char() {
                return is_whitespace(c);
            }

            return false;
        }

        fn ready() {
            ready = true;
        }

        fn ret(t: Token) {
            token = Some(t, token_info);
            ready();
        }

        fn flush_line() {
            self.reader.flush_line();
            if !buffer.is_empty() {
                ret(Token::literal(buffer));
            }
        }

        while let Some(c) = self.reader.current_char() {

            match c {
                DOUBLE_QUOTE if self.stringing => {
                    self.stringing = false;
                    ret(Token::string(buffer));
                },
                DOUBLE_QUOTE if !self.stringing => {
                    self.stringing = true;
                },
                _ if self.stringing => {
                    buffer.push(c);
                },
                COMMENT => self.reader.flush_line(),
                _ if is_whitespace(c) => {},
                LIST_START => ret(Token::list_start()),
                LIST_END => ret(Token::list_end()),
                VECTOR_START => ret(Token::vector_start()),
                VECTOR_END => ret(Token::vector_end()),
                MAP_START => ret(Token::map_start()),
                MAP_END => ret(Token::map_end()),
                DISPATCH if is_next(BANG) && is_prev_whitespace() => self.reader.flush_line(),
                DISPATCH if is_next(LIST_START) => ret(Token::function_start()),
                DISPATCH if is_next(MAP_START) => ret(Token::set_start()),
                _ => {
                    buffer.push(c);

                    if let Some(n) = self.reader.next_char() {
                        match n {
                            LIST_END | MAP_END | VECTOR_END | COMMENT => ret(Token::literal(buffer)),
                            _ if is_whitespace(n) => ret(Token::literal(buffer))
                        }
                    }
                    else {
                        ret(Token::literal(buffer));
                    }
                }
            }

            self.reader.pop();
            if ready { break; }
        }

        return token;
    }
}

pub fn tokenize(str: String) -> TokenStream<StringScanner> {
    let reader = StringScanner::new(&str);
    TokenStream {reader: reader, stringing: false}
}

pub fn tokenize_file(path: String) -> TokenStream<LineScanner> {
    let reader = LineScanner::from_file(&path);
    TokenStream {reader: reader, stringing: false}
}

pub fn tokenize_stream<R: BufRead>(buf_reader: R) -> TokenStream<LineScanner> {
    let reader = LineScanner::from_buffer(buf_reader);
    TokenStream {reader: reader, stringing: false}
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
