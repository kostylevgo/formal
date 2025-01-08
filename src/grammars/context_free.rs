use std::collections::HashMap;
use std::io::BufRead;
use std::fmt::{Formatter, Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonTerminal {
    id: usize
}

impl NonTerminal {
    pub fn new(id: usize) -> Self {
        Self {id}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol {
    Non(NonTerminal),
    Terminal(char)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammarRule {
    pub left: NonTerminal,
    pub right: Vec<Symbol>
}

impl GrammarRule {
    pub fn new(left: NonTerminal, right: Vec<Symbol>) -> GrammarRule {
        Self {
            left,
            right
        }
    }
}

#[derive(Clone, Debug)]
struct NonTerminalManager {
    count: usize,
    letter_codes: HashMap<char, usize>
}

#[derive(Clone, Debug)]
pub struct Grammar {
    manager: NonTerminalManager,
    rules: HashMap<NonTerminal, Vec<GrammarRule>>,
    pub starting: NonTerminal
}

impl NonTerminalManager {
    pub fn new() -> NonTerminalManager {
        Self {
            count: 0,
            letter_codes: HashMap::new()
        }
    }

    pub fn get_count(&self) -> usize {
        self.count
    }

    pub fn new_non_terminal(&mut self) -> NonTerminal {
        self.count += 1;
        NonTerminal::new(self.count - 1)
    }

    fn to_non_terminal(&mut self, non_terminal: char) -> NonTerminal {
        NonTerminal::new(
            if self.letter_codes.contains_key(&non_terminal) {
                *self.letter_codes.get(&non_terminal).unwrap()
            } else {
                self.letter_codes.insert(non_terminal, self.count);
                self.count += 1;
                self.count - 1
            }
        )
    }

    pub fn to_rule(&mut self, str: &String) -> Option<GrammarRule> {
        let str_trimmed = str.chars().filter(|x| !x.is_whitespace()).collect::<String>();
        let mut iter = str_trimmed.split("->");
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
        let right_part = right_part.chars().map(|x| if x.is_ascii_uppercase() {Symbol::Non(self.to_non_terminal(x))} else {Symbol::Terminal(x)}).collect();
        Some(GrammarRule::new(left_part, right_part))
    }
}

impl Grammar {
    pub fn new() -> Grammar {
        let mut manager = NonTerminalManager::new();
        let starting = manager.new_non_terminal();
        let mut rules = HashMap::new();
        rules.insert(starting, Vec::new());
        Self {
            manager,
            rules,
            starting
        }
    }

    pub fn add_rule(&mut self, rule: GrammarRule) {
        self.rules.get_mut(&rule.left).unwrap().push(rule);
    }

    pub fn get_rules(&self, starts_with: NonTerminal) -> &Vec<GrammarRule> {
        self.rules.get(&starts_with).unwrap()
    }

    pub fn get_non_terminals(&self) -> Vec<NonTerminal> {
        self.rules.iter().map(|x| *x.0).collect()
    }

    pub fn new_non_terminal(&mut self) -> NonTerminal {
        let res = self.manager.new_non_terminal();
        self.rules.insert(res, Vec::new());
        res
    }

    pub fn add_exclusive_starting_non_terminal(&mut self) -> &GrammarRule {
        let start = self.starting;
        let start_prime = self.new_non_terminal();
        let new_rule = GrammarRule::new(start_prime, vec![Symbol::Non(start)]);
        self.add_rule(new_rule);
        self.starting = start_prime;
        &self.rules.get(&start_prime).unwrap()[0]
    }

    pub fn read(source: &mut impl BufRead) -> Result<Grammar, String> {
        let mut read_line = || -> Option<String> {
            let mut buf = String::new();
            match source.read_line(&mut buf) {
                Err(_) => None,
                Ok(_) => Some(buf)
            }
        };
        let first_line = read_line();
        let cnt_rules = match first_line {
            None => {
                return Err("rules count not found".to_string());
            }
            Some(str) => {
                match String::from(str).split_whitespace().next_back() {
                    None => return Err("rules count not found".to_string()),
                    Some(x) => {
                        x.parse::<usize>()
                    }
                }
            }
        };
        let cnt_rules = match cnt_rules {
            Err(_) => return Err("rules count not found".to_string()),
            Ok(value) => value
        };
        for _ in 0..2 {
            match read_line() {
                None => return Err("alphabet not found".to_string()),
                _ => ()
            }
        }
        let mut manager = NonTerminalManager::new();
        let rules = (0..cnt_rules).map(|_| {
            read_line().and_then(|x| manager.to_rule(&x))
        }).collect::<Vec<Option<GrammarRule>>>();
        let starting_char = match read_line() {
            None => return Err("starting non-terminal not found".to_string()),
            Some(some) => {
                if some.len() == 0 {
                    return Err("starting non-terminal not found".to_string());
                }
                some.chars().nth(0).unwrap()
            }
        };
        let starting = manager.to_non_terminal(starting_char);
        let mut rules_map = HashMap::new();
        for id in 0..manager.get_count() {
            rules_map.insert(NonTerminal::new(id), Vec::new());
        }
        let mut res = Self {
            manager,
            rules: rules_map,
            starting
        };
        for (index, rule) in rules.into_iter().enumerate() {
            match rule {
                None => return Err(format!("error while parsing rule {}", index + 1)),
                Some(rule) => res.add_rule(rule),
            }
        }
        Ok(res)
    }
}

impl Display for NonTerminal {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if self.id < 26 {write!(formatter, "{}", (('A' as u8 + self.id as u8) as char).to_string())} else {write!(formatter, "<{}>", self.id)}
    }
}

impl Display for GrammarRule {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} -> |", self.left)?;
        for symbol in self.right.iter() {
            match symbol {
                Symbol::Non(non_terminal) => {
                    write!(formatter, "{}", non_terminal)?;
                }
                Symbol::Terminal(ch) => {
                    write!(formatter, "{}", ch.to_string())?;
                }
            }
        }
        write!(formatter, "|")
    }
}

impl Display for Grammar {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "starting: {}\n", self.starting)?;
        let count = self.manager.get_count();
        for id in 0..count {
            let left = NonTerminal::new(id);
            for rule in self.rules.get(&left).unwrap().iter() {
                write!(formatter, "{}\n", rule)?;
            }
        }
        std::fmt::Result::Ok(())
    }
}

pub trait ParsingAlgorithm : Sized {
    fn fit(grammar: Grammar) -> Option<Self>;
    fn predict(&self, word: &String) -> bool;
}
