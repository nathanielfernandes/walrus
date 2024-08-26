#![allow(dead_code)]
pub mod ast;
pub mod compiler;
mod pool;

use pool::Pool;

// Warren Abstract Machine
// wam-book - l0

// Atom, represented as in interned string, in an atom pool.
pub type Atom = u16;
pub type Reg = u8;

#[derive(Debug, Clone, Copy)]
enum HeapCell {
    // Variable Cell, denoted as <REF, k> where k is an index into the heap.
    // Facilitates variable binding by allowing multiple cells to point to the same heap cell.
    // An unbound variable is represented by a reference to itself.
    Ref(u16),
    //
    // Structure Cell, denoted as <STR, k> where k is an index into the heap,
    // where the representation of the functor is stored.
    // Structures 'f(t1, t2, ..., tn)' are layed out in memory as:
    // <STR, k> <FUN, n, a> <t1> <t2> ... <tn> | requiring n+2 cells.
    // The first 2 cells are the structure cell which points to the functor cell.
    // This functor cell may not be contiguous with the structure cell,
    Str(u16),
    //
    // Functor Cell, denoted as <FUN, n, a> where n is the name of the functor,
    // and a is the arity of the functor.
    // The functor cell may not be contiguous with the structure cell,
    // and may be shared among multiple structures.
    // The functor cell is always contiguously followed by the arguments,
    // such that if HEAP[k] = <FUN, n, a> then HEAP[k+1] = t1, HEAP[k+2] = t2, ..., HEAP[k+n] = tn.
    Fun { name: Atom, arity: u8 },
}

pub struct Machine {
    // global registers
    h: usize, // address of the next free heap cell

    heap: Vec<HeapCell>,
    symbols: Pool<String, u16>,
}
