use std::fmt::{Display, Formatter};

use crate::data_structures::graph::DerivedFromGraph;

pub trait AutomatonLike<T>: DerivedFromGraph<T> {
    fn get_starting(&self) -> usize;
    fn is_accepting(&self, vertex: usize) -> bool;
}

pub trait DisplayableLikeAutomaton<T: Display>: AutomatonLike<T> {
    fn display_like_automaton(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match write!(f, "starting: {}, accepting: ", self.get_starting()) {
            Err(some) => return Result::Err(some),
            _ => ()
        }
        for state in 0..self.size() {
            if self.is_accepting(state) {
                match write!(f, "{}, ", state) {
                    Err(some) => return Result::Err(some),
                    _ => ()
                }
            }
        }
        write!(f, "\n{}", self.get_graph())
    }
}
