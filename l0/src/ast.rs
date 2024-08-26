pub enum Term {
    Atom(String),
    Variable(String),
    Clause { functor: String, args: Vec<Term> }, // structure
}
