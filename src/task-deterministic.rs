use std::{collections::HashMap, fmt::{Display, Error, Formatter}};

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
struct Transition<T> {
    to: usize,
    str: T,
}

#[derive(Clone, Debug)]
struct NonDeterministicAutomaton {
    transitions: Vec<Vec<Transition<String>>>,
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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

    fn remove_equal_transitions(&mut self, state: usize) {
        self.transitions[state].sort();
        self.transitions[state].dedup();
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
            result.transitions[color[state]].extend(transitions.into_iter().map(|transition| Transition::<String> {to: color[transition.to], str: transition.str}));
        }
        for state in 0..result.size() {
            result.remove_equal_transitions(state);
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
                aut.remove_equal_transitions(state);
            }
        }
        for state in 0..self.size() {
            if !used[state] {
                propagation_dfs(self, &mut used, &epsilon_transitions, state);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct DeterministicAutomaton {
    transitions: Vec<Vec<Transition<char>>>,
    starting: usize,
    is_accepting: Vec<bool>
}

impl DeterministicAutomaton {
    fn from(mut aut: NonDeterministicAutomaton) -> Self {
        aut.remove_multi_character_transitions();
        aut = aut.compress_epsilon_cycles();
        aut.remove_epsilon_transitions();
        let mut state_decoder: HashMap<Vec<bool>, usize> = HashMap::new();
        let mut is_accepting = Vec::<bool>::new();
        let mut transitions = Vec::<Vec<Transition<char>>>::new();
        let mut register_state = |state: &Vec<bool>, transitions: &mut Vec<Vec<Transition<char>>>| -> (usize, bool) {
            if state_decoder.contains_key(state) {
                (state_decoder[state], false)
            } else {
                let new_num = state_decoder.len();
                transitions.push(Vec::<Transition<char>>::new());
                is_accepting.push(state.iter().enumerate().any(|(state, is_present)| *is_present && aut.is_accepting[state]));
                state_decoder.insert(state.clone(), new_num);
                (new_num, true)
            }
        };
        let starting_state: Vec<bool> = (0..aut.size()).map(|state| state == aut.starting).collect();
        let starting_state_index = register_state(&starting_state, &mut transitions).0;
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
                for transition in aut.transitions[old_state].iter() {
                    options.push(transition.str.chars().nth(0).unwrap());
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
                    for transition in aut.transitions[old_state].iter() {
                        if transition.str.chars().nth(0).unwrap() == *char {
                            next_state[transition.to] = true;
                        }
                    }
                }
                let res = register_state(&next_state, &mut transitions);
                transitions[state_index].push(Transition::<char> {to: res.0, str: *char});
                if res.1 {
                    bfs_vec.push((res.0, next_state));
                }
            }
        }
        Self {
            transitions,
            starting: 0,
            is_accepting,
        }
    }

    fn size(&self) -> usize {
        self.is_accepting.len()
    }
}

impl Display for NonDeterministicAutomaton {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match write!(f, "size: {}, starting: {}, accepting: ", self.size(), self.starting) {
            Err(some) => return Result::Err(some),
            _ => ()
        }
        for state in 0..self.size() {
            if self.is_accepting[state] {
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
        for state in 0..self.size() {
            for transition in self.transitions[state].iter() {
                match write!(f, "<{}, {}> -> {}\n", state, transition.str, transition.to) {
                    Err(some) => return Result::Err(some),
                    _ => (),
                }
            }
        }
        Result::Ok(())
    }
}

impl Display for DeterministicAutomaton {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match write!(f, "size: {}, starting: {}, accepting: ", self.size(), self.starting) {
            Err(some) => return Result::Err(some),
            _ => ()
        }
        for state in 0..self.size() {
            if self.is_accepting[state] {
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
        for state in 0..self.size() {
            for transition in self.transitions[state].iter() {
                match write!(f, "<{}, {}> -> {}\n", state, transition.str, transition.to) {
                    Err(some) => return Result::Err(some),
                    _ => (),
                }
            }
        }
        Result::Ok(())
    }
}

fn main() {
    let mut aut = NonDeterministicAutomaton::new(2, 0);
    aut.add_transition(0, 0, "ab".to_string());
    aut.add_transition(0, 1, String::new());
    aut.add_transition(1, 1, "aab".to_string());
    aut.mark_as_accepting(1);
    let aut = DeterministicAutomaton::from(aut);
    println!("{}\n", aut);
}