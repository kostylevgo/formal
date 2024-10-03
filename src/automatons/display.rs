use std::fmt::{Display, Error, Formatter};
use crate::automatons::*;
use non_deterministic::{Transition, NonDeterministicAutomaton};
use deterministic::DeterministicAutomaton;

fn print_automaton_like(size: usize, starting: usize, is_accepting: &Vec<bool>,
        transitions: &Vec<Vec<Transition<impl Display>>>, f: &mut Formatter) -> Result<(), Error> {
    match write!(f, "size: {}, starting: {}, accepting: ", size, starting) {
        Err(some) => return Result::Err(some),
        _ => ()
    }
    for state in 0..size {
        if is_accepting[state] {
            match write!(f, "{}, ", state) {
                Err(some) => return Result::Err(some),
                _ => ()
            }
        }
    }
    match write!(f, "\n") {
        Err(some) => return Result::Err(some),
        _ => ()
    }
    for state in 0..size {
        for transition in transitions[state].iter() {
            match write!(f, "<{}, {}> -> {}\n", state, transition.str, transition.to) {
                Err(some) => return Result::Err(some),
                _ => (),
            }
        }
    }
    Result::Ok(())
}

impl Display for NonDeterministicAutomaton {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        print_automaton_like(self.size(), self.starting, &self.is_accepting, &self.transitions, f)
    }
}

impl Display for DeterministicAutomaton {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        print_automaton_like(self.size(), self.starting, &self.is_accepting, &self.transitions, f)
    }
}
