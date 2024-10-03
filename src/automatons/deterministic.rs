use crate::non_deterministic::{Transition, NonDeterministicAutomaton};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DeterministicAutomaton {
    pub transitions: Vec<Vec<Transition<char>>>,
    pub starting: usize,
    pub is_accepting: Vec<bool>
}

impl DeterministicAutomaton {
    pub fn from(mut aut: NonDeterministicAutomaton) -> Self {
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

    pub fn size(&self) -> usize {
        self.is_accepting.len()
    }

    pub fn check_word(&self, str: &String) -> bool {
        let mut state = self.starting;
        for char in str.chars() {
            let mut found = false;
            for transition in self.transitions[state].iter() {
                if transition.str == char {
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
        aut.add_transition(1, 0, String::new());
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
