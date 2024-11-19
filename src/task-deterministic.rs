mod data_structures;

use data_structures::*;
use graph::DerivedFromGraphMut;
use non_deterministic::NonDeterministicAutomaton;
use deterministic::DeterministicAutomaton;

fn main() {
    let mut aut = NonDeterministicAutomaton::with_size(7, 0);
    // enter automaton data by hand
    let a = || "a".to_string();
    let b = || "b".to_string();
    aut.add_edge(0, 1, a());
    aut.add_edge(1, 1, a());
    aut.add_edge(1, 2, a());
    aut.add_edge(1, 4, b());
    aut.add_edge(2, 3, b());
    aut.add_edge(3, 1, a());
    aut.add_edge(4, 5, b());
    aut.add_edge(5, 4, a());
    aut.add_edge(4, 6, a());
    aut.add_edge(6, 1, a());
    aut.mark_as_accepting(0);
    aut.mark_as_accepting(1);
    aut.mark_as_accepting(3);
    aut.mark_as_accepting(6);
    // stop entering
    let aut = DeterministicAutomaton::from(aut);
    println!("{}\n", aut);
    let check = aut.check_word(&"aab".to_string());
    println!("{}", check);
}
