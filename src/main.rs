#![allow(dead_code, unused_variables, unused_imports)]

mod interpreter;
mod parser;

mod runtime;

extern crate getopts;
extern crate llvm_sys as llvm;

use getopts::Options;
use std::env;
use std::io::stdin;
use ::runtime::Environment;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("l", "llvm", "LLVM demo");
    opts.optopt("s", "str", "parse expression from string", "FANCY EXPR");
    opts.optflag("p", "pipe", "parse stdin");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

     if matches.opt_present("l") {
        println!("LLVM demo");
        runtime::demo();
    }

    let mut expressions = Vec::<parser::Expression>::new();

    if let Some(s) =  matches.opt_str("s") {
        expressions = parser::parse_string(s);
    }

    if expressions.len() == 0 && matches.opt_present("p") {
        let stdin = stdin();
        expressions = parser::parse_buffer(stdin.lock());
    }

    if expressions.len() == 0 && !matches.free.is_empty() {
        let path = matches.free[0].clone();
        expressions = parser::parse_file(path);
    }

    if expressions.len() == 0 {
        print_usage(&program, opts);
        return;
    }
    else {
        println!("{:?}", expressions);

        let mut env = ::runtime::LLVMEnvironment::new();
        let last = env.eval_all(expressions);

        //env.llvm_dump();
    }

    return;
}
