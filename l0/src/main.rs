use l0::{ast::Term, compiler::Compiler};

fn main() {
    let mut c = Compiler::new();

    // p(Z, h(Z, W), f(W))
    let query = Term::Clause {
        functor: String::from("p"),
        args: vec![
            Term::Variable(String::from("Z")),
            Term::Clause {
                functor: String::from("h"),
                args: vec![
                    Term::Variable(String::from("Z")),
                    Term::Variable(String::from("W")),
                ],
            },
            Term::Clause {
                functor: String::from("f"),
                args: vec![Term::Variable(String::from("W"))],
            },
        ],
    };

    c.compile_query(&query);

    let program = c.program();
    let symbols = c.symbols();

    println!("query: p(Z, h(Z, W), f(W))");
    println!("program:");
    for instr in program {
        println!("{}", instr.fmt(symbols));
    }
}
