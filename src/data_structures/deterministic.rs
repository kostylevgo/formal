use std::collections::HashMap;

use crate::automaton_like::{AutomatonLike, DisplayableLikeAutomaton};
use crate::non_deterministic::NonDeterministicAutomaton;
use crate::graph::{Graph, DerivedFromGraph};

#[derive(Clone, Debug)]
pub struct DeterministicAutomaton {
    graph: Graph<char>,
    starting: usize,
    is_accepting: Vec<bool>
}

impl DerivedFromGraph<char> for DeterministicAutomaton {
    fn get_graph(&self) -> &Graph<char> {
        &self.graph
    }
}

impl AutomatonLike<char> for DeterministicAutomaton {
    fn is_accepting(&self, vertex: usize) -> bool {
        self.is_accepting[vertex]
    }

    fn get_starting(&self) -> usize {
        self.starting
    }
}

impl DisplayableLikeAutomaton<char> for DeterministicAutomaton {}

impl std::fmt::Display for DeterministicAutomaton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_like_automaton(f)
    }
}

impl DeterministicAutomaton {
    pub fn from(mut aut: NonDeterministicAutomaton) -> Self {
        aut.remove_multi_character_transitions();
        aut = aut.compress_epsilon_cycles();
        aut.remove_epsilon_transitions();
        let mut state_decoder: HashMap<Vec<bool>, usize> = HashMap::new();
        let mut is_accepting = Vec::<bool>::new();
        let mut graph = Graph::<char>::new();
        let mut register_state = |state: &Vec<bool>,
                graph: &mut Graph<char>| -> (usize, bool) {
            if state_decoder.contains_key(state) {
                (state_decoder[state], false)
            } else {
                let new_num = state_decoder.len();
                graph.add_vertex();
                is_accepting.push(state.iter().enumerate().any(|(state, is_present)|
                    *is_present && aut.is_accepting(state)
                ));
                state_decoder.insert(state.clone(), new_num);
                (new_num, true)
            }
        };
        let starting_state: Vec<bool> = (0..aut.size()).map(|state| state == aut.starting).collect();
        let starting_state_index = register_state(&starting_state, &mut graph).0;
        let mut bfs_vec = Vec::<(usize, Vec<bool>)>::new();
        bfs_vec.push((starting_state_index, starting_state));
        let mut index = 0;
        while index < bfs_vec.len() {
            let (state_index, state) = bfs_vec[index].clone();
            index += 1;
            let mut options = Vec::<char>::new();
            for old_state in 0..aut.size() {
                if !state[old_state] {
                    continue;
                }
                for transition in aut.get_edges(old_state).iter() {
                    options.push(transition.value.chars().nth(0).unwrap());
                }
            }
            options.sort();
            options.dedup();
            for char in options.iter() {
                let mut next_state = vec![false; aut.size()];
                for old_state in 0..aut.size() {
                    if !state[old_state] {
                        continue;
                    }
                    for transition in aut.get_edges(old_state).iter() {
                        if transition.value.chars().nth(0).unwrap() == *char {
                            next_state[transition.to] = true;
                        }
                    }
                }
                let res = register_state(&next_state, &mut graph);
                graph.add_edge(state_index, res.0, *char);
                if res.1 {
                    bfs_vec.push((res.0, next_state));
                }
            }
        }
        Self {
            graph,
            starting: 0,
            is_accepting,
        }
    }

    pub fn check_word(&self, str: &String) -> bool {
        let mut state = self.starting;
        for char in str.chars() {
            let mut found = false;
            for transition in self.get_edges(state).iter() {
                if transition.value == char {
                    found = true;
                    state = transition.to;
                    break;
                }
            }
            if !found {
                return false;
            }
        }
        self.is_accepting[state]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_structures::graph::DerivedFromGraphMut;

    use crate::non_deterministic::tests::make_testing_aut;

    #[test]
    fn test_basic() {
        let aut = make_testing_aut();
        let aut = DeterministicAutomaton::from(aut);

        assert!(aut.check_word(&"".to_string()));
        assert!(aut.check_word(&"ab".to_string()));
        assert!(aut.check_word(&"aab".to_string()));
        assert!(aut.check_word(&"ababab".to_string()));
        assert!(aut.check_word(&"aabaabaabaab".to_string()));
        assert!(aut.check_word(&"ababaab".to_string()));

        assert!(!aut.check_word(&"a".to_string()));
        assert!(!aut.check_word(&"b".to_string()));
        assert!(!aut.check_word(&"abaa".to_string()));

        assert!(!aut.check_word(&"aabab".to_string()))
    }

    #[test]
    fn test_cyclic() {
        let mut aut = make_testing_aut();
        aut.add_edge(1, 0, String::new());
        let aut = DeterministicAutomaton::from(aut);

        assert!(aut.check_word(&"".to_string()));
        assert!(aut.check_word(&"ab".to_string()));
        assert!(aut.check_word(&"aab".to_string()));
        assert!(aut.check_word(&"ababab".to_string()));
        assert!(aut.check_word(&"aabaabaabaab".to_string()));
        assert!(aut.check_word(&"ababaab".to_string()));

        assert!(!aut.check_word(&"a".to_string()));
        assert!(!aut.check_word(&"b".to_string()));
        assert!(!aut.check_word(&"abaa".to_string()));

        assert!(aut.check_word(&"aabab".to_string()))
    }
}
