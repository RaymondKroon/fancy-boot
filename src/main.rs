mod parser;

extern crate getopts;
use getopts::Options;
use std::env;
use std::io::stdin;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [FILE] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("s", "", "parse expression from file", "FANCY EXPR");
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

    if let Some(s) =  matches.opt_str("s") {
        let expressions = parser::parse_string(s);
        println!("{:?}", expressions);

        return;
    }

    if matches.opt_present("p") {
        let stdin = stdin();
        let expressions = parser::parse_buffer(stdin.lock());
        println!("{:?}", expressions);

        return;
    }

    let path = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    let expressions = parser::parse_file(path);
    println!("{:?}", expressions);

    return;
}
