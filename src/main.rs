//#![allow(unused_imports)]
//#![allow(unused_variables)]
//#![allow(dead_code)]

use std::str::Chars;

type Token = String;

struct TokenStream<'a> {
    rest: Chars<'a>,
    current_char: Option<char>,
    next_char: Option<char>,
    stringing: bool
}

const LIST: (&'static str, &'static str) = ("(",")");
const MAP: (&'static str, &'static str) = ("{","}");
const VECTOR: (&'static str, &'static str) = ("[","]");
const STRING: (&'static str, &'static str) = ("\"","\"");

const START_CHARS: [char; 3] = ['(', '{', '['];
const END_CHARS: [char; 3] = [')', '}', ']'];

const QUOTE: char = '"';
const DISPATCH : char = '#';

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

fn tokenize<'a>(str: &'a str) -> TokenStream<'a> {
    let mut rest = str.chars();
    let current = rest.next();
    let next = rest.next();
    TokenStream {rest: rest, current_char: current, next_char: next, stringing: false}
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Form {
    Literal(String),
    String(String),
    List(Vec<Form>),
    Vector(Vec<Form>),
    Map(Vec<Form>),
    Dispatch(String, Vec<Form>),
    None
}

#[derive(PartialEq, Eq, Debug, Clone)]
enum Outer {
    List,
    Vector,
    Map,
    Dispatch,
    None
}

struct FormStream<'rf> {
    tokens: &'rf mut Iterator<Item = Token>,
    current_token: Option<Token>,
    next_token: Option<Token>,
    outer: Outer
}

impl<'rf> FormStream<'rf> {

    fn read_inner<'a>(&mut self, outer: Outer)
                                -> Vec<Form> {
        self.read_next();

        let old_outer = self.outer.clone();
        self.outer = outer;


        let inner: Vec<Form> = self.collect();

        // copy back;
        self.outer = old_outer;
        return inner;
    }

    fn read_next (&mut self) {
        self.current_token = self.next_token.clone();
        self.next_token = self.tokens.next();
    }

    fn assert_token(&mut self, expected_token: &'static str) {
        self.assert_token_string(String::from(expected_token));
    }

    fn assert_token_string (&mut self, expected_token: String) {
        if let Some(ref t) = self.current_token {
            if *t != expected_token {
                panic!("expected {:?}, got {:?}", expected_token, t);
            }
        }
        else {
            panic!("expected {:?}, got nothing", expected_token);
        }
    }
}

impl<'a> Iterator for FormStream<'a> {
    type Item = Form;

    fn next(&mut self) -> Option<Form> {
        let mut form = Form::None;

        if let Some(t) = self.current_token.clone() {
            if t == LIST.0 {
                form = Form::List(self.read_inner(Outer::List));
                self.assert_token(LIST.1);
            }
            else if t == VECTOR.0 {
                form = Form::Vector(self.read_inner( Outer::Vector));
                self.assert_token(VECTOR.1);
            }
            else if t == MAP.0 {
                form = Form::Map(self.read_inner(Outer::Map));
                self.assert_token(MAP.1);
            }
            else if t == STRING.0 {

                self.read_next();
                if let Some(t) = self.current_token.clone() {
                    if t == STRING.0 {
                        form = Form::String(String::from(""));
                    }
                    else {
                        self.read_next();
                        form = Form::String(t);
                    }
                }
                else {
                    panic!("Code ends with opening string");
                }

                self.assert_token(STRING.0);
            }
            else if t.starts_with(DISPATCH) {

                let mut inner: Vec<Form> = Vec::new();

                let mut rev_t = t.chars().collect::<Vec<char>>();
                rev_t.reverse();
                let last : &char = rev_t.first().unwrap();

                if START_CHARS.contains(last) {
                    let stop_at = match last {
                        &'(' => LIST.1,
                        &'[' => VECTOR.1,
                        &'{' => MAP.1,
                        _ => unreachable!("dispatch read error")
                    };

                    inner = self.read_inner(Outer::Dispatch);
                    self.assert_token(stop_at);
                }

                form = Form::Dispatch(t, inner);
            }
            else if t != LIST.1 && t != VECTOR.1 && t != MAP.1 {
                form = Form::Literal(t);
            }
            else if self.outer == Outer::Dispatch &&
                (t == LIST.1 || t == VECTOR.1 || t == MAP.1 || t ==  STRING.1)
            {
                //nothing
            }
            else if (self.outer != Outer::List && t == LIST.1) ||
                (self.outer != Outer::Vector && t == VECTOR.1) ||
                (self.outer != Outer::Map && t == MAP.1)
            {

                panic!("read error: unmatched closing token for {:?}", self.outer);
            }
            else { // nothing
            }

            if form != Form::None {
                self.read_next();
            }
         }


        if form == Form::None {
            return None::<Form>;
        }
        else {
            return Some(form);
        }
    }

}

fn read<'rf>(tokens: &'rf mut Iterator<Item = Token>) -> FormStream<'rf> {
    let current = tokens.next();
    let next = tokens.next();
    FormStream{tokens: tokens, current_token: current, next_token: next, outer: Outer::None}
}

#[derive(Debug)]
enum Expression {
    Symbol(String),
    Number(String),
    String(String),
    SExpression(Vec<Expression>)
}

trait HasForm {
    fn give_form(&mut self) -> &mut Form;
}

impl HasForm for Form {
    fn give_form(&mut self) -> &mut Form {
        return self;
    }
}

struct ExpressionStream<'rf, T: HasForm> {
    forms: &'rf mut Iterator<Item = T>,
}

fn prepend<T>(item: T, mut v: Vec<T>) -> Vec<T> {
    v.insert(0, item);
    v
}

fn symbol(name: &'static str) -> Expression {
    Expression::Symbol(String::from(name))
}

fn dispatch(value: String, inner: Vec<Form>) -> Expression {

    match value.as_ref() {
        "#{" => Expression::SExpression(prepend(symbol("set"), parse_vec(inner))),
        _ => panic!("Unknow dispatch value {}", value)
    }
}

impl<'a, T: HasForm> Iterator for ExpressionStream<'a, T> {
    type Item = Expression;

    fn next(&mut self) -> Option<Expression> {
        //None::<Expression>
        if let Some(mut t) = self.forms.next() {
            match t.give_form().clone() {
                Form::List(inner) =>
                    return Some(Expression::SExpression(parse_vec(inner))),
                Form::Vector(inner) =>
                    return Some(Expression::SExpression(
                        prepend(symbol("vector"),(parse_vec(inner))))),
                Form::Map(inner) =>
                    return Some(Expression::SExpression(
                        prepend(symbol("hash-map"),(parse_vec(inner))))),
                Form::Literal(value) => {
                    let chars = value.chars().collect::<Vec<char>>();
                    if chars[0].is_numeric() ||
                        (chars.len() > 1 && chars[0] == '-' && chars[1].is_numeric()) {
                        return Some(Expression::Number(value));
                    }
                    else {
                        return Some(Expression::Symbol(value));
                    }
                },
                Form::String(value) =>
                    return Some(Expression::String(value)),
                Form::Dispatch(value, inner) =>
                    return Some(dispatch(value, inner)),
               _ => unreachable!("expression error")
           }
        }
        else {
            None::<Expression>
        }
    }
}

fn parse_vec<T>(forms: Vec<T>) -> Vec<Expression>
    where T: HasForm
{
    return parse(&mut forms.into_iter()).collect();
}

fn parse<'rf, I, T>(forms: &'rf mut I) -> ExpressionStream<'rf, T>
    where I: Iterator<Item = T>, T: HasForm
{
    ExpressionStream{forms: forms}
}

fn main() {
    //let tokens: TokenStream = tokenize("(test,,, 12234dd 2 3) { 1 2} [12, a] #{:a :b} [] #fancy[] (()) #\"regexp\" \"meerdere woorden, met daartussen spaties enzo()\" \"(sde");

   // println!("{:?}", tokens.collect::<Vec<String>>());

    // let mut read_tokens = tokenize("#{1 [2] 3}(4 5 6)\"Test      texkt\"");
    // let form_test = read(&mut read_tokens);

    // println!("{:?}", form_test.collect::<Vec<Form>>());

    let mut parse_tokens = tokenize("(+ 1 2 -3)");
    let mut parse_form = read(&mut parse_tokens);
    let expression_test = parse(&mut parse_form);

    println!("{:?}", expression_test.collect::<Vec<Expression>>());
}
