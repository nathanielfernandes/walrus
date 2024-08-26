use hashbrown::HashMap;

use crate::{ast::Term, pool::Pool, Atom, Reg};

#[derive(Debug)]
pub enum Instruction {
    // Query Instructions
    PutStructure { functor: Atom, arity: Reg, reg: Reg },
    SetVariable { reg: Reg },
    SetValue { reg: Reg },

    // Program Instructions
    GetStructure { functor: Atom, arity: Reg, reg: Reg },
    UnifyVariable { reg: Reg },
    UnifyValue { reg: Reg },
}

impl Instruction {
    pub fn fmt(&self, symbols: &Pool<String, Atom>) -> String {
        match self {
            Instruction::PutStructure {
                functor,
                arity,
                reg,
            } => {
                let functor = symbols.get(*functor).expect("functor not found");
                format!("put_structure {}/{}, X{}", functor, arity, reg + 1)
            }
            Instruction::SetVariable { reg } => format!("set_variable X{}", reg + 1),
            Instruction::SetValue { reg } => format!("set_value X{}", reg + 1),
            Instruction::GetStructure {
                functor,
                arity,
                reg,
            } => {
                let functor = symbols.get(*functor).expect("functor not found");
                format!("get_structure {}/{}, X{}", functor, arity, reg + 1)
            }
            Instruction::UnifyVariable { reg } => format!("unify_variable X{}", reg + 1),
            Instruction::UnifyValue { reg } => format!("unify_value X{}", reg + 1),
        }
    }
}

struct AllocatedVar {
    reg: Reg,
    seen: bool,
}

enum TermIR<'a> {
    StartClause {
        functor: Atom,
        args: &'a Vec<Term>,
        reg: Reg,
    },
    EndClause {
        functor: Atom,
        args: &'a Vec<Term>,
        reg: Reg,
        max_reg: Reg,
    },
    Atom {
        atom: Atom,
        reg: Reg,
    },
}

pub struct Compiler {
    symbols: Pool<String, Atom>,         // symbol table
    allocs: HashMap<Atom, AllocatedVar>, // variable allocations

    program: Vec<Instruction>,
}

impl Compiler {
    // register X1 is always allocated to the outermost term
    const X1: Reg = 0;

    pub fn new() -> Self {
        Compiler {
            symbols: Pool::new(),
            allocs: HashMap::new(),

            program: Vec::new(),
        }
    }

    fn lookup_symbol(&self, atom: Atom) -> Option<&String> {
        self.symbols.get(atom)
    }

    fn intern(&mut self, symbol: &String) -> Atom {
        self.symbols.add_ref(symbol)
    }

    fn reset(&mut self) {
        self.allocs.clear();
        self.program.clear();
    }

    pub fn symbols(&self) -> &Pool<String, Atom> {
        &self.symbols
    }
    pub fn program(&self) -> &Vec<Instruction> {
        &self.program
    }

    fn instr(&mut self, instr: Instruction) {
        self.program.push(instr);
    }

    fn alloc(&mut self, name: Atom, reg: Reg) {
        self.allocs.insert(name, AllocatedVar { reg, seen: false });
    }

    fn is_allocated(&self, name: Atom) -> bool {
        self.allocs.contains_key(&name)
    }

    fn mark_seen(&mut self, name: Atom) -> bool {
        if let Some(var) = self.allocs.get_mut(&name) {
            if var.seen {
                return true;
            }

            var.seen = true;
        }

        return false;
    }

    fn compile_query_clause(&mut self, functor: Atom, args: &Vec<Term>) {
        let mut ir_stack = Vec::new();

        ir_stack.push(TermIR::StartClause {
            functor,
            args,
            reg: Self::X1,
        });

        let mut max_reg: u8 = Self::X1;

        while let Some(ir) = ir_stack.pop() {
            match ir {
                TermIR::StartClause { functor, args, reg } => {
                    ir_stack.push(TermIR::EndClause {
                        functor,
                        args,
                        reg,
                        max_reg,
                    });

                    let mut cur_reg: u8 = max_reg;

                    // allocate registers for variables, and count the number of registers
                    for term in args {
                        if let &Term::Variable(ref atom) = term {
                            let atom = self.intern(atom);

                            if !self.is_allocated(atom) {
                                cur_reg += 1;

                                self.alloc(atom, cur_reg);
                            }
                        } else {
                            cur_reg += 1;
                        }
                    }

                    max_reg = cur_reg;

                    // post order so iter in reverse
                    for term in args.iter().rev() {
                        match term {
                            Term::Atom(atom) => {
                                ir_stack.push(TermIR::Atom {
                                    atom: self.intern(atom),
                                    reg: cur_reg,
                                });
                                cur_reg -= 1;
                            }
                            Term::Variable(_) => {
                                cur_reg -= 1;
                            }
                            Term::Clause { functor, args } => {
                                ir_stack.push(TermIR::StartClause {
                                    functor: self.intern(functor),
                                    args,
                                    reg: cur_reg,
                                });
                                cur_reg -= 1;
                            }
                        }
                    }
                }
                TermIR::EndClause {
                    functor,
                    args,
                    reg,
                    max_reg: mr,
                } => {
                    let instr = Instruction::PutStructure {
                        functor,
                        arity: args.len() as Reg,
                        reg,
                    };
                    self.instr(instr);

                    let mut cur_reg = mr + 1;

                    for term in args {
                        match term {
                            Term::Variable(ref atom) => {
                                let atom = self.intern(atom);

                                let v_reg =
                                    self.allocs.get(&atom).expect("variable not allocated").reg;

                                let instr = if self.mark_seen(atom) {
                                    Instruction::SetValue { reg: v_reg }
                                } else {
                                    Instruction::SetVariable { reg: v_reg }
                                };
                                self.instr(instr);

                                if v_reg == cur_reg {
                                    cur_reg += 1;
                                }
                            }
                            _ => {
                                let instr = Instruction::SetValue { reg: cur_reg };
                                self.instr(instr);

                                cur_reg += 1;
                            }
                        }

                        max_reg = cur_reg - 1;
                    }
                }
                TermIR::Atom { atom, reg } => {
                    // an atom is a structure with 0 arity
                    let instr = Instruction::PutStructure {
                        functor: atom,
                        arity: 0,
                        reg,
                    };
                    self.instr(instr);
                }
            }
        }
    }

    pub fn compile_query(&mut self, term: &Term) {
        match term {
            Term::Atom(atom) => {
                let atom = self.intern(atom);
                let instr = Instruction::PutStructure {
                    functor: atom,
                    arity: 0,
                    reg: Self::X1,
                };
                self.instr(instr);
            }
            Term::Variable(_) => {
                let instr = Instruction::SetVariable { reg: Self::X1 };
                self.instr(instr);
            }
            Term::Clause { functor, args } => {
                let functor = self.intern(functor);
                self.compile_query_clause(functor, &args);
            }
        }
    }
}
