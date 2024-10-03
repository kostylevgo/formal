mod automatons;

use automatons::*;
use non_deterministic::NonDeterministicAutomaton;
use deterministic::DeterministicAutomaton;

fn main() {
    let mut aut = NonDeterministicAutomaton::new(2, 0);
    // enter automaton data by hand
    aut.add_transition(0, 0, "a".to_string());
    aut.add_transition(0, 1, "ab".to_string());
    aut.mark_as_accepting(1);
    // stop entering
    let aut = DeterministicAutomaton::from(aut);
    println!("{}", aut);
    let check = aut.check_word(&"aab".to_string());
    println!("{}", check);
}
