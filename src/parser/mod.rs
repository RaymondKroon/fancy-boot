mod tokenizer;
mod reader;

use std::io::{BufRead, Read};

use self::reader::Form;
use self::tokenizer as tok;

const LIST: (&'static str, &'static str) = ("(",")");
const MAP: (&'static str, &'static str) = ("{","}");
const VECTOR: (&'static str, &'static str) = ("[","]");
const STRING: (&'static str, &'static str) = ("\"","\"");

const LIST_CHARS: (char, char) = ('(',')');
const VECTOR_CHARS: (char, char) = ('[',']');
const MAP_CHARS: (char, char) = ('{','}');
const START_CHARS: [char; 3] = ['(', '{', '['];
const END_CHARS: [char; 3] = [')', '}', ']'];

const QUOTE: char = '"';
const DISPATCH : char = '#';
const COMMENT: char = ';';

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Symbol(String),
    Number(String),
    String(String),
    SExpression(Vec<Expression>),
    Params(Vec<Expression>)
}

pub struct ExpressionStream<'rf> {
    forms: &'rf mut Iterator<Item = Form>,
}

fn prepend<T>(item: T, mut v: Vec<T>) -> Vec<T> {
    v.insert(0, item);
    v
}

pub fn symbol(name: &'static str) -> Expression {
    Expression::Symbol(String::from(name))
}

pub fn number(name: &'static str) -> Expression {
    Expression::Number(String::from(name))
}

fn dispatch(value: String, inner: Vec<Form>) -> Expression {

    match value.as_ref() {
        "#{" => Expression::SExpression(prepend(symbol("set"), parse_vec(inner))),
        "#[" => Expression::Params(parse_vec(inner)),
        _ => panic!("Unknow dispatch value {}", value)
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
