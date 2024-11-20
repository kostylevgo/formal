use super::automaton_like::{AutomatonLike, DisplayableLikeAutomaton};
use super::graph::{Graph, DerivedFromGraph, DerivedFromGraphMut};
use super::single_accepting::SingleAcceptingAutomaton;

#[derive(Clone, Debug)]
pub struct NonDeterministicAutomaton {
    graph: Graph<String>,
    pub starting: usize,
    is_accepting: Vec<bool>,
}

impl DerivedFromGraph<String> for NonDeterministicAutomaton {
    fn get_graph(&self) -> &Graph<String> {
        &self.graph
    }
}

impl DerivedFromGraphMut<String> for NonDeterministicAutomaton {
    fn get_graph_mut(&mut self) -> &mut Graph<String> {
        &mut self.graph
    }

    fn add_vertex(&mut self) -> usize {
        self.is_accepting.push(false);
        self.graph.add_vertex()
    }
}

impl AutomatonLike<String> for NonDeterministicAutomaton {
    fn is_accepting(&self, vertex: usize) -> bool {
        self.is_accepting[vertex]
    }

    fn get_starting(&self) -> usize {
        self.starting
    }
}

impl DisplayableLikeAutomaton<String> for NonDeterministicAutomaton {}

impl std::fmt::Display for NonDeterministicAutomaton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_like_automaton(f)
    }
}

impl NonDeterministicAutomaton {
    pub fn with_size(size: usize, starting: usize) -> Self {
        Self {
            graph: Graph::with_size(size),
            starting,
            is_accepting: vec![false; size],
        }
    }

    pub fn from_graph(graph: Graph<String>, starting: usize) -> Self {
        let size = graph.size();
        Self {
            graph,
            starting,
            is_accepting: vec![false; size],
        }
    }

    pub fn into_single_accepting(mut self) -> SingleAcceptingAutomaton {
        enum SearchResult {
            None,
            Some(usize),
            Several,
        }
        let search_for_accepting = (0..self.size()).fold(SearchResult::None, |res, x| {
            if self.is_accepting(x) {
                match res {
                    SearchResult::None => SearchResult::Some(x),
                    _ => SearchResult::Several
                }
            } else {
                res
            }
        });
        match search_for_accepting {
            SearchResult::Some(accepting) => {
                SingleAcceptingAutomaton::from_graph(self.graph, self.starting, accepting)
            }
            _ => {
                let new_accepting = self.add_vertex();
                for state in 0..self.size() {
                    if self.is_accepting[state] {
                        self.add_edge(state, new_accepting, String::new());
                    }
                }
                SingleAcceptingAutomaton::from_graph(self.graph, self.starting, new_accepting)
            }
        }
    }

    pub fn mark_as_accepting(&mut self, num: usize) {
        self.is_accepting[num] = true;
    }

    #[allow(dead_code)]
    pub fn unmark_as_accepting(&mut self, num: usize) {
        self.is_accepting[num] = false;
    }

    pub fn remove_multi_character_transitions(&mut self) {
        for state in 0..self.size() {
            for transition_index in 0..self.get_edges(state).len() {
                let transition = &mut self.get_edges_mut(state)[transition_index];
                if transition.value.len() == 0 {
                    continue;
                }
                let mut first_char = transition.value.chars().nth(0).unwrap().to_string();
                std::mem::swap(&mut first_char, &mut transition.value);
                let str = first_char;
                let mut last_state = transition.to;
                for (index, char) in str.char_indices().rev() {
                    if index == 0 {
                        self.get_edges_mut(state)[transition_index].to = last_state;
                    } else {
                        let next_state = self.add_vertex();
                        self.add_edge(next_state, last_state, char.to_string());
                        last_state = next_state;
                    }
                }
            }
        }
    }

    pub fn compress_epsilon_cycles(self) -> Self {
        let epsilon_transitions = self.get_epsilon_transitions();
        let strongly_connected_components = epsilon_transitions.kosaraju();
        let mut new_accepting = vec![false; self.size()];
        for (i, val) in self.is_accepting.into_iter().enumerate() {
            new_accepting[strongly_connected_components[i]] |= val;
        }
        Self {
            graph: self.graph.compress(&strongly_connected_components),
            starting: strongly_connected_components[self.starting],
            is_accepting: new_accepting
        }
    }

    /*
    should be only called if epsilon transitions are not cyclic
    if they are cyclic, call compress_epsilon_cycles before
     */
    pub fn remove_epsilon_transitions(&mut self) { 
        let epsilon_transitions = self.get_epsilon_transitions();
        for transitions in self.get_edges_list_mut() {
            transitions.retain(|transition| transition.value.len() != 0);
        }
        let mut used = vec![false; self.size()];
        fn propagation_dfs<U>(aut: &mut NonDeterministicAutomaton, used: &mut Vec<bool>,
                epsilon_transitions: &Graph<U>, state: usize) {
            used[state] = true;
            for next_state in epsilon_transitions.get_edges(state).iter() {
                if !used[next_state.to] {
                    propagation_dfs(aut, used, epsilon_transitions, next_state.to);
                }
                aut.is_accepting[state] |= aut.is_accepting[next_state.to];
                let mut copying = aut.get_edges(next_state.to).clone();
                aut.get_edges_mut(state).append(&mut copying);
                aut.graph.remove_equal_edges(state);
            }
        }
        for state in 0..self.size() {
            if !used[state] {
                propagation_dfs(self, &mut used, &epsilon_transitions, state);
            }
        }
    }

    fn get_epsilon_transitions(&self) -> Graph<String> {
        let mut res = self.graph.clone();
        res.retain(|x| x.2.len() == 0);
        res
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // makes automaton for regular expression (ab)*(aab)*
    pub fn make_testing_aut() -> NonDeterministicAutomaton {
        let mut aut = NonDeterministicAutomaton::with_size(2, 0);
        aut.add_edge(0, 0, "ab".to_string());
        aut.add_edge(0, 1, String::new());
        aut.add_edge(1, 1, "aab".to_string());
        aut.mark_as_accepting(1);
        aut
    }

    #[test]
    fn test_basic() {
        let mut aut = NonDeterministicAutomaton::with_size(1, 0);
        assert!(aut.starting == 0);
        assert!(aut.size() == 1);
        aut.add_vertex();
        assert!(aut.size() == 2);

        assert!(aut.get_edges(0).len() == 0);
        aut.add_edge(0, 1, "aba".to_string());
        assert!(aut.get_edges(0).len() == 1);
        aut.remove_edge(0, 1, &"aba".to_string());
        assert!(aut.get_edges(0).len() == 0);

        assert!(!aut.is_accepting[0]);
        aut.mark_as_accepting(0);
        assert!(aut.is_accepting[0]);
        aut.mark_as_accepting(0);
        assert!(aut.is_accepting[0]);
        aut.unmark_as_accepting(0);
        assert!(!aut.is_accepting[0]);
    }

    #[test]
    #[should_panic]
    fn test_panic1() {
        let mut aut = NonDeterministicAutomaton::with_size(2, 0);
        aut.mark_as_accepting(2);
    }

    #[test]
    #[should_panic]
    fn test_panic2() {
        let mut aut = NonDeterministicAutomaton::with_size(3, 0);
        aut.add_edge(0, 3, "a".to_string());
    }

    #[test]
    fn test_remove_multi_character_transitions() {
        let mut aut = make_testing_aut();
        aut.remove_multi_character_transitions();
        for state in 0..aut.size() {
            for transition in aut.get_edges(state).iter() {
                assert!(transition.value.len() <= 1);
            }
        }
    }

    #[test]
    fn test_remove_epsilon_transitions() {
        let mut aut = make_testing_aut();
        aut.remove_epsilon_transitions();
        for state in 0..aut.size() {
            for transition in aut.get_edges(state).iter() {
                assert!(transition.value.len() >= 1);
            }
        }
    }

    #[test]
    fn test_cyclic() {
        let mut aut = make_testing_aut();
        aut.add_edge(1, 0, String::new());
        let aut = NonDeterministicAutomaton::compress_epsilon_cycles(aut);
        assert!(aut.size() == 1);
    }
}
