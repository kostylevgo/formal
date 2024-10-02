#[derive(Clone, Debug)]
struct Transition {
    to: usize,
    str: String,
}

#[derive(Clone, Debug)]
struct NonDeterministicAutomaton {
    transitions: Vec<Vec<Transition>>,
    starting: usize,
    is_accepting: Vec<bool>,
}

impl NonDeterministicAutomaton {
    fn new(size: usize, starting: usize) -> Self {
        Self {
            transitions: vec![Vec::new(); size],
            starting,
            is_accepting: vec![false; size],
        }
    }

    fn add_transition(&mut self, from: usize, to: usize, str: String) {
        self.transitions[from].push(Transition {to, str});
    }

    fn remove_transition(&mut self, from: usize, to: usize, str: String) {
        let found = self.transitions[from].iter().position(|x| x.to == to && x.str == str);
        match found {
            Some(index) => {self.transitions[from].remove(index);},
            None => ()
        }
    }

    fn mark_as_accepting(&mut self, num: usize) {
        self.is_accepting[num] = true;
    }

    fn unmark_as_accepting(&mut self, num: usize) {
        self.is_accepting[num] = false;
    }

    fn add_state(&mut self) -> usize {
        self.transitions.push(Vec::new());
        self.is_accepting.push(false);
        self.is_accepting.len() - 1
    }

    fn size(&self) -> usize {
        self.is_accepting.len()
    }

    fn remove_multi_character_transitions(&mut self) {
        for state in 0..self.size() {
            for transition_index in 0..self.transitions[state].len() {
                let transition = &mut self.transitions[state][transition_index];
                if transition.str.len() == 0 {
                    continue;
                }
                let mut first_char = transition.str.chars().nth(0).unwrap().to_string();
                std::mem::swap(&mut first_char, &mut transition.str);
                let str = first_char;
                let mut last_state = transition.to;
                for (index, char) in str.char_indices().rev() {
                    if index == 0 {
                        self.transitions[state][transition_index].to = last_state;
                    } else {
                        let next_state = self.add_state();
                        self.add_transition(next_state, last_state, char.to_string());
                        last_state = next_state;
                    }
                }
            }
        }
    }

    fn get_epsilon_transitions(&self) -> Vec<Vec<usize>> {
        self.transitions.iter().map(|transitions| {
            let res = transitions.iter().filter_map(|transition| {
                if transition.str.len() == 0 {
                    Some(transition.to)
                } else {
                    None
                }
            }).collect();
            res
        }).collect()
    }

    fn compress_epsilon_cycles(self) -> Self {
        let epsilon_transitions = self.get_epsilon_transitions();
        let mut inverse_epsilon_transitions = vec![Vec::<usize>::new(); self.size()];
        for state in 0..self.size() {
            for next_state in epsilon_transitions[state].iter() {
                inverse_epsilon_transitions[*next_state].push(state);
            }
        }
        let mut used = vec![false; self.size()];
        let mut pseudo_top_sort = Vec::<usize>::new();
        fn pseudo_top_sort_dfs(graph: &Vec<Vec<usize>>, used: &mut Vec<bool>, result: &mut Vec<usize>, state: usize) {
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
                pseudo_top_sort_dfs(&inverse_epsilon_transitions, &mut used, &mut pseudo_top_sort, state);
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
        let mut result = NonDeterministicAutomaton::new(color_counter, color[self.starting]);
        for (state, transitions) in self.transitions.into_iter().enumerate() {
            result.is_accepting[color[state]] |= self.is_accepting[state];
            result.transitions[color[state]].extend(transitions);
        }
        result
    }

    fn remove_epsilon_transitions(&mut self) {
        let epsilon_transitions = self.get_epsilon_transitions();
        for transitions in self.transitions.iter_mut() {
            transitions.retain(|transition| transition.str.len() != 0);
        }
        let mut used = vec![false; self.size()];
        fn propagation_dfs(aut: &mut NonDeterministicAutomaton, used: &mut Vec<bool>, epsilon_transitions: &Vec<Vec<usize>>, state: usize) {
            used[state] = true;
            for next_state in epsilon_transitions[state].iter() {
                if !used[*next_state] {
                    propagation_dfs(aut, used, epsilon_transitions, *next_state);
                }
                aut.is_accepting[state] |= aut.is_accepting[*next_state];
                let mut copying = aut.transitions[*next_state].clone();
                aut.transitions[state].append(&mut copying);
            }
        }
        for state in 0..self.size() {
            if !used[state] {
                propagation_dfs(self, &mut used, &epsilon_transitions, state);
            }
        }
    }
}

fn main() {

}