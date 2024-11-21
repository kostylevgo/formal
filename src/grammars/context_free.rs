use std::{collections::HashMap, iter};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonTerminal {
    id: usize
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    NonTerminal(NonTerminal),
    Terminal(char)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammarRule {
    pub left: NonTerminal,
    pub right: Vec<Symbol>
}

#[derive(Clone, Debug)]
struct NonTerminalManager {
    count: usize,
    letter_codes: HashMap<char, usize>
}

// #[derive(Clone, Debug)]
// struct RuleStorage {
//     rules: HashMap<NonTerminal, Vec<Vec<Symbol>>>
// }

#[derive(Clone, Debug)]
struct Grammar {
    manager: NonTerminalManager,
    storage: HashMap<NonTerminal, Vec<GrammarRule>>
}

impl NonTerminalManager {
    pub fn new() -> NonTerminalManager {
        Self {
            count: 0,
            letter_codes: HashMap::new()
        }
    }

    fn to_non_terminal(&mut self, non_terminal: char) -> NonTerminal {
        NonTerminal {
            id: if self.letter_codes.contains_key(&non_terminal) {
                *self.letter_codes.get(&non_terminal).unwrap()
            } else {
                self.letter_codes.insert(non_terminal, self.count);
                self.count += 1;
                self.count - 1
            }
        }
    }

    pub fn new_non_terminal(&mut self) -> NonTerminal {
        self.count += 1;
        NonTerminal {
            id: self.count - 1
        }
    }

    pub fn to_rule(&mut self, str: &String) -> Option<GrammarRule> {
        let mut iter = str.split("->");
        let left_part: String = match iter.next() {
            None => return None,
            Some(str) => {
                str.chars().filter(|x| !x.is_whitespace()).collect()
            }
        };
        let right_part: String = match iter.next() {
            None => return None,
            Some(str) => {
                str.chars().filter(|x| !x.is_whitespace()).collect()
            }
        };
        if left_part.len() != 1 {
            return None;
        }
        let left_part = self.to_non_terminal(left_part.chars().nth(0).unwrap());
        let right_part = right_part.chars().map(|x| if x.is_ascii_lowercase() {Symbol::Terminal(x)} else {Symbol::NonTerminal(self.to_non_terminal(x))}).collect();
        Some(GrammarRule {
            left: left_part,
            right: right_part
        })
    }
}

// impl RuleStorage {
//     fn make_vec(&mut self, non_terminal: NonTerminal) {
//         if !self.rules.contains_key(&rule.left) {
//             self.rules.insert(rule.left, Vec::new());
//         }
//     }

//     pub fn new() -> RuleStorage {
//         Self {
//             rules: HashMap::new()
//         }
//     }

//     pub fn add_rule(&mut self, rule: GrammarRule) {
//         self.make_vec(rule.left);
//         self.rules.get_mut(&rule.left).unwrap().push(rule.right);
//     }

//     pub fn iter_rules_with_left(&mut self, left: NonTerminal) -> impl Iterator {
//         self.make_vec(left);
//         self.rules.get(&left).unwrap().iter().map(|x| GrammarRule {
//             left: left,
//             right: x
//         })
//     }

//     pub fn iter(&self) -> impl Iterator {
        
//     }
// }