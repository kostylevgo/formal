use super::{automaton_like::AutomatonLike, graph::{DerivedFromGraph, DerivedFromGraphMut, Graph}, non_deterministic::NonDeterministicAutomaton, reg_exp::RegularExpression};
use super::automaton_like::DisplayableLikeAutomaton;

#[derive(Clone, Debug)]
pub struct SingleAcceptingAutomaton {
    graph: Graph<String>,
    pub starting: usize,
    pub accepting: usize,
}

impl DerivedFromGraph<String> for SingleAcceptingAutomaton {
    fn get_graph(&self) -> &Graph<String> {
        &self.graph
    }
}

impl DerivedFromGraphMut<String> for SingleAcceptingAutomaton {
    fn get_graph_mut(&mut self) -> &mut Graph<String> {
        &mut self.graph
    }
}

impl AutomatonLike<String> for SingleAcceptingAutomaton {
    fn is_accepting(&self, vertex: usize) -> bool {
        vertex == self.accepting
    }

    fn get_starting(&self) -> usize {
        self.starting
    }
}

impl DisplayableLikeAutomaton<String> for SingleAcceptingAutomaton {}

impl std::fmt::Display for SingleAcceptingAutomaton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_like_automaton(f)
    }
}

impl SingleAcceptingAutomaton {
    pub fn with_size(size: usize, starting: usize, accepting: usize) -> SingleAcceptingAutomaton {
        SingleAcceptingAutomaton {
            graph: Graph::with_size(size),
            starting,
            accepting,
        }
    }

    pub fn from_graph(graph: Graph<String>, starting: usize, accepting: usize) -> SingleAcceptingAutomaton {
        SingleAcceptingAutomaton {
            graph,
            starting,
            accepting,
        }
    }

    pub fn from_regular_expression(reg: RegularExpression) -> SingleAcceptingAutomaton {
        use RegularExpression::*;
        match reg {
            Zero => {
                Self::with_size(2, 0, 1)
            }
            One => {
                Self::with_size(1, 0, 0)
            }
            Letter(ch) => {
                let mut res = Self::with_size(2, 0, 1);
                res.add_edge(0, 1, ch.to_string());
                res
            }
            Sum((lhs, rhs)) => {
                let mut left_aut = Self::from_regular_expression(*lhs);
                let right_aut = Self::from_regular_expression(*rhs);
                let (right_start, right_accept) = left_aut.concat_automaton(&right_aut);
                let new_start = left_aut.add_vertex();
                let new_accept = left_aut.add_vertex();
                left_aut.add_edge(new_start, left_aut.starting, String::new());
                left_aut.add_edge(new_start, right_start, String::new());
                left_aut.add_edge(left_aut.accepting, new_accept, String::new());
                left_aut.add_edge(right_accept, new_accept, String::new());
                left_aut.starting = new_start;
                left_aut.accepting = new_accept;
                left_aut
            }
            Concatenation((lhs, rhs)) => {
                let mut left_aut = Self::from_regular_expression(*lhs);
                let right_aut = Self::from_regular_expression(*rhs);
                let (right_start, right_accept) = left_aut.concat_automaton(&right_aut);
                left_aut.add_edge(left_aut.accepting, right_start, String::new());
                left_aut.accepting = right_accept;
                left_aut
            }
            Iteration(val) => {
                let mut aut = Self::from_regular_expression(*val);
                let new_start_accept = aut.add_vertex();
                aut.add_edge(aut.accepting, new_start_accept, String::new());
                aut.add_edge(new_start_accept, aut.starting, String::new());
                aut.starting = new_start_accept;
                aut.accepting = new_start_accept;
                aut
            }
        }
    }

    fn concat_automaton(&mut self, other: &SingleAcceptingAutomaton) -> (usize, usize) {
        let old_size = self.size();
        for _ in 0..other.size() {
            self.add_vertex();
        }
        for i in 0..other.size() {
            for transition in other.get_edges(i).iter() {
                self.add_edge(old_size + i, old_size + transition.to, transition.value.clone());
            }
        }
        (old_size + other.starting, old_size + other.accepting)
    }

    pub fn into_non_deterministic(self) -> NonDeterministicAutomaton {
        let size = self.size();
        let mut res = NonDeterministicAutomaton::from_graph(self.graph, self.starting);
        res.mark_as_accepting(self.accepting);
        res
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{automaton_like::DisplayableLikeAutomaton, non_deterministic::tests::make_testing_aut};

    #[test]
    fn test_already_single() {
        let not_det_aut = make_testing_aut();
        let single_aut = not_det_aut.clone().into_single_accepting();
        assert!(not_det_aut.size() == single_aut.size());
        assert!(not_det_aut.is_accepting(single_aut.accepting));
    }

    #[test]
    fn test_multiple() {
        let mut not_det_aut = make_testing_aut();
        not_det_aut.mark_as_accepting(0);
        let single_aut = not_det_aut.clone().into_single_accepting();
        assert!(not_det_aut.size() + 1 == single_aut.size());
    }

    #[test]
    fn test_two_way_conversion() {
        let old_det_aut = make_testing_aut();
        let single_aut = old_det_aut.clone().into_single_accepting();
        let new_det_aut = single_aut.into_non_deterministic();
        assert!(new_det_aut.size() == old_det_aut.size());
        assert!(new_det_aut.get_edges_list() == old_det_aut.get_edges_list());
        for i in 0..new_det_aut.size() {
            assert!(new_det_aut.is_accepting(i) == old_det_aut.is_accepting(i));
        }
    }
}
