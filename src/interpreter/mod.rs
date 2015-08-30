use std::str::FromStr;

use parser;
use parser::Expression;
use parser::Expression::{Symbol, Number, SExpression};
use parser::Expression::String as Str;

//TODO: interpreter implementation will fail on recursion
pub fn interpret(input: Vec<Expression>) -> Expression
{
    let mut result: Expression = SExpression(vec!());
    for i in input {
        match i {
            SExpression(expression) => result = sexpression(&expression),
            _ => result = SExpression(vec!()),
        };
    }
    result
}

fn lookup_symbol(name: String) -> Box<FnMut(Vec<Expression>) -> Expression> {
    match name.as_ref() {
        "+" => Box::new(fn_plus),
        "-" => Box::new(fn_min),
        _ => Box::new(|v: Vec<Expression>| parser::number("3"))
    }
}

fn fn_plus(expr: Vec<Expression>) -> Expression {
     let mut acc: i64 = 0;
     for i in expr {
         match i {
             Number(number) => acc += FromStr::from_str(&number).unwrap_or(0),
             SExpression(exprs) => match sexpression(&(exprs.clone())) {
                    Number(number) => acc += FromStr::from_str(&number).unwrap_or(0),
                    _ => (),
                 },
             _ => (),
         }
     };
     Number(acc.to_string())
}

fn fn_min(expr: Vec<Expression>) -> Expression {
     let mut acc: i64 = match expr[0].clone() {
         Number(number) => FromStr::from_str(&number).unwrap_or(0),
         _ => 0,
     };
     let rest = expr[1..].to_vec();
     for i in rest {
         match i {
             Number(number) => acc -= FromStr::from_str(&number).unwrap_or(0),
             SExpression(exprs) => match sexpression(&(exprs.clone())) {
                    Number(number) => acc -= FromStr::from_str(&number).unwrap_or(0),
                    _ => (),
                 },
             _ => (),
         }
     };
     Number(acc.to_string())
}

fn sexpression(expr: &Vec<Expression>) -> Expression {
    let error = parser::symbol("error");
    let error_str = String::from("error");
    let symbol = match expr.first().unwrap_or(&error) {
        &Symbol(ref symbol) => symbol,
        _ => &error_str,
    };
    let rest: Vec<Expression> = expr[1..].to_vec();
    (*lookup_symbol(symbol.clone()))(rest)
}

#[test]
fn interpret_min_file() {
    let expressions = parser::parse_file(String::from("resources/interpreter/min.fc"));
    assert_eq!(interpret(vec!(expressions[0].clone())), expressions[1]);
}

#[test]
fn interpret_plus_file() {
    let expressions = parser::parse_file(String::from("resources/interpreter/plus.fc"));
    assert_eq!(interpret(vec!(expressions[0].clone())), expressions[1]);
}

#[test]
fn interpret_empty() {
    assert_eq!(interpret(vec!()), SExpression(vec!()));
}

#[test]
fn interpret_plus_1_2() {
    assert_eq!(
        interpret(
            vec!(SExpression(
                vec!(parser::symbol("+"),
                     parser::number("1"),
                     parser::number("2"))))),
            parser::number("3")

        )
}

#[test]
fn interpret_plus_10_20() {
    assert_eq!(
        interpret(
            vec!(SExpression(
                vec!(parser::symbol("+"),
                     parser::number("10"),
                     parser::number("20"))))),
            parser::number("30")

        )
}
