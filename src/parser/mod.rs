mod tokenizer;

use std::io::{BufRead, Read};

//use super::util;
use self::tokenizer as tok;
use self::tokenizer::{Token, TokenInfo};

const LIST: (&'static str, &'static str) = ("(",")");
const MAP: (&'static str, &'static str) = ("{","}");
const VECTOR: (&'static str, &'static str) = ("[","]");
const STRING: (&'static str, &'static str) = ("\"","\"");

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    List(List),
    Number(Number),
    Map(Map),
    Regex(Regex),
    Set(Set),
    String(Stringy),
    Symbol(Symbol),
    Vector(Vector)
}

pub fn symbol(str: String) { Expression::Symbol(Symbol {name: str}) }
pub fn number(str: String) { Expression::Number(Number {value: str}) }
pub fn string(str: String) { Expression::String(Stringy {value: str}) }


pub struct List { inner: Vec<Expression> }
pub struct Number {value: String}
pub struct Map { inner: Vec<Expression> }
pub struct Regex {value: String}
pub struct Set { inner: Vec<Expression> }
pub struct Stringy {value: String}
pub struct Symbol {name: String}
pub struct Vector { inner: Vec<Expression> }

pub struct ExpressionStream<'rf> {
    tokens: &'rf mut Iterator<Item = (Token, TokenInfo)>,
    current_token: Option<(Token, TokenInfo)>,
    next_token: Option<(Token, TokenInfo)>,
    outer: Some<Token>
}

fn prepend<T>(item: T, mut v: Vec<T>) -> Vec<T> {
    v.insert(0, item);
    v
}

fn dispatch(value: String, inner: Vec<Form>) -> Expression {

    match value.as_ref() {
        "#{" => Expression::SExpression(prepend(symbol("set"), parse_vec(inner))),
        "#[" => Expression::Params(parse_vec(inner)),
        _ => panic!("Unknow dispatch value {}", value)
    }
}

impl<'rf> ExpressionStream<'rf> {

    fn read_inner<'a>(&mut self, outer: Some<Token>)
                                -> Vec<Expression> {
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


impl<'a> Iterator for ExpressionStream<'a> {
    type Item = Expression;

    fn next(&mut self) -> Option<Expression> {

        //None::<Expression>
        if let Some(t) = self.forms.next() {
            match t.clone() {
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

fn parse_vec(forms: Vec<Form>) -> Vec<Expression>
{
    return parse(&mut forms.into_iter()).collect();
}

fn parse<'rf>(forms: &'rf mut Iterator<Item = Form>) -> ExpressionStream<'rf>
{
    ExpressionStream{forms: forms}
}

pub fn parse_string(s: String) -> Vec<Expression> {
    let mut tokens = tok::tokenize(s);
    let mut forms = reader::read(&mut tokens);

    let expressions = parse(&mut forms);
    expressions.collect::<Vec<Expression>>()
}

pub fn parse_file(path: String) -> Vec<Expression> {
    let mut tokens = tok::tokenize_file(path);
    let mut forms = reader::read(&mut tokens);

    let expression = parse(&mut forms);
    expression.collect::<Vec<Expression>>()
}

pub fn parse_buffer<R: BufRead>(reader: R) -> Vec<Expression> {
    let mut tokens = tok::tokenize_stream(reader);
    let mut forms = reader::read(&mut tokens);

    let expression = parse(&mut forms);
    expression.collect::<Vec<Expression>>()
}
