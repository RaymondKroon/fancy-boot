mod parser;

extern crate getopts;
use getopts::Options;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    //let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "", "parse expression from file", "FANCY EXPR");
    opts.optopt("f", "file", "parse file", "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if let Some(s) =  matches.opt_str("s") {
        let expressions = parser::parse_string(s);
        println!("{:?}", expressions);

        return;
    }

    if let Some(s) =  matches.opt_str("f") {
        let expressions = parser::parse_file(s);
        println!("{:?}", expressions);

        return;
    }

    return;
}
