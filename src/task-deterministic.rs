mod data_structures;

use data_structures::*;
use deterministic::DeterministicAutomaton;
use non_deterministic::NonDeterministicAutomaton;
use graph::DerivedFromGraphMut;

pub fn make_testing_aut() -> NonDeterministicAutomaton {
    let mut aut = NonDeterministicAutomaton::with_size(2, 0);
    aut.add_edge(0, 0, "ab".to_string());
    aut.add_edge(0, 1, String::new());
    aut.add_edge(1, 1, "aab".to_string());
    aut.mark_as_accepting(1);
    aut
}

fn main() {
    let mut aut = make_testing_aut();
    aut.remove_multi_character_transitions();
    println!("{}\n", aut);
    aut = aut.compress_epsilon_cycles();
    println!("{}\n", aut);
    // stop entering
    let aut = DeterministicAutomaton::from(aut);
    println!("{}\n", aut);
    println!("{}\n", aut);
    let check = aut.check_word(&"aab".to_string());
    println!("{}", check);
}
