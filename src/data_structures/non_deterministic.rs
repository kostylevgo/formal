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
        let mut inverse_epsilon_transitions = vec![Vec::<usize>::new(); self.size()];
        for state in 0..self.size() {
            for next_state in epsilon_transitions[state].iter() {
                inverse_epsilon_transitions[*next_state].push(state);
            }
        }
        let mut used = vec![false; self.size()];
        let mut pseudo_top_sort = Vec::<usize>::new();
        fn pseudo_top_sort_dfs(graph: &Vec<Vec<usize>>, used: &mut Vec<bool>,
                result: &mut Vec<usize>, state: usize) {
            used[state] = true;
            for next_state in graph[state].iter() {
                if !used[*next_state] {
                    pseudo_top_sort_dfs(graph, used, result, *next_state);
                }
            }
            result.push(state);
        }
        for state in 0..self.size() {
            if !used[state] {
                pseudo_top_sort_dfs(&inverse_epsilon_transitions, &mut used,
                    &mut pseudo_top_sort, state);
            }
        }
        let mut color = vec![usize::MAX; self.size()];
        let mut color_counter = 0;
        fn coloring_dfs(graps: &Vec<Vec<usize>>, colors: &mut Vec<usize>, state: usize, color: usize) {
            colors[state] = color;
            for next_state in graps[state].iter() {
                if colors[*next_state] == usize::MAX {
                    coloring_dfs(graps, colors, *next_state, color);
                }
            }
        }
        for state in pseudo_top_sort.iter().rev() {
            if color[*state] == usize::MAX {
                coloring_dfs(&epsilon_transitions, &mut color, *state, color_counter);
                color_counter += 1;
            }
        }
        let mut result = NonDeterministicAutomaton::with_size(color_counter, color[self.starting]);
        for (state, transitions) in self.graph.into_edges().into_iter().enumerate() {
            result.is_accepting[color[state]] |= self.is_accepting[state];
            for edge in transitions.into_iter() {
                result.graph.add_edge(color[state], color[edge.to], edge.value);
            }
        }
        for state in 0..result.size() {
            result.remove_equal_transitions(state);
        }
        result
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
        fn propagation_dfs(aut: &mut NonDeterministicAutomaton, used: &mut Vec<bool>,
                epsilon_transitions: &Vec<Vec<usize>>, state: usize) {
            used[state] = true;
            for next_state in epsilon_transitions[state].iter() {
                if !used[*next_state] {
                    propagation_dfs(aut, used, epsilon_transitions, *next_state);
                }
                aut.is_accepting[state] |= aut.is_accepting[*next_state];
                let mut copying = aut.get_edges(*next_state).clone();
                aut.get_edges_mut(state).append(&mut copying);
                aut.remove_equal_transitions(state);
            }
        }
        for state in 0..self.size() {
            if !used[state] {
                propagation_dfs(self, &mut used, &epsilon_transitions, state);
            }
        }
    }

    fn get_epsilon_transitions(&self) -> Vec<Vec<usize>> {
        self.graph.get_edges_list().iter().map(|transitions| {
            let res = transitions.iter().filter_map(|transition| {
                if transition.value.len() == 0 {
                    Some(transition.to)
                } else {
                    None
                }
            }).collect();
            res
        }).collect()
    }

    fn remove_equal_transitions(&mut self, state: usize) {
        self.get_edges_mut(state).sort();
        self.get_edges_mut(state).dedup();
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
