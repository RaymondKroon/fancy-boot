use super::tokenizer::Token;
use super::{LIST,VECTOR,MAP,STRING,DISPATCH,START_CHARS};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Form {
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

pub struct FormStream<'rf> {
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

pub fn read<'rf>(tokens: &'rf mut Iterator<Item = Token>) -> FormStream<'rf> {
    let current = tokens.next();
    let next = tokens.next();
    FormStream{tokens: tokens, current_token: current, next_token: next, outer: Outer::None}
}
