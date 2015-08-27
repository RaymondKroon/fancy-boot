mod parser;

fn main() {
    //let tokens: TokenStream = tokenize("(test,,, 12234dd 2 3) { 1 2} [12, a] #{:a :b} [] #fancy[] (()) #\"regexp\" \"meerdere woorden, met daartussen spaties enzo()\" \"(sde");

   // println!("{:?}", tokens.collect::<Vec<String>>());

    // let mut read_tokens = tokenize("#{1 [2] 3}(4 5 6)\"Test      texkt\"");
    // let form_test = read(&mut read_tokens);

    // println!("{:?}", form_test.collect::<Vec<Form>>());

    let expressions = parser::parse_string(String::from("(+ 1 2 -3)"));

    println!("{:?}", expressions);
}
