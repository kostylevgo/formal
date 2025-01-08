use std::collections::HashMap;

use super::context_free::*;
use super::earley::GrammarSituation;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct First {
    has_epsilon: bool,
    characters: Vec<char>,
}

fn merge_vecs<T: Ord + Clone>(lhs: &Vec<T>, rhs: &Vec<T>) -> Vec<T> {
    let mut lhs_index = 0;
    let mut rhs_index = 0;
    let mut result = Vec::<T>::new();
    while lhs_index < lhs.len() || rhs_index < rhs.len() {
        if lhs_index == lhs.len() || rhs_index < rhs.len() && lhs[lhs_index] > rhs[rhs_index] {
            result.push(rhs[rhs_index].clone());
            rhs_index += 1;
        } else {
            result.push(lhs[lhs_index].clone());
            lhs_index += 1;
        }
    }
    result.dedup();
    result
}

impl First {
    fn concatenate(&mut self, other: &First) {
        if self.has_epsilon {
            self.has_epsilon &= other.has_epsilon;
            self.characters = merge_vecs(&self.characters, &other.characters);
        }
    }

    fn unite(&mut self, other: &First) {
        self.has_epsilon |= other.has_epsilon;
        self.characters = merge_vecs(&self.characters, &other.characters);
    }

    fn from_empty_string() -> First {
        Self {
            has_epsilon: true,
            characters: Vec::new(),
        }
    }

    fn from_char(symbol: char) -> First {
        Self {
            has_epsilon: false,
            characters: vec![symbol]
        }
    }

    fn new() -> First {
        Self {
            has_epsilon: false,
            characters: Vec::new(),
        }
    }

    fn get_first(non_terminal_firsts: &HashMap<NonTerminal, First>, string: impl Iterator<Item = Symbol>) -> First {
        let mut result = First::from_empty_string();

        for symbol in string {
            let cur_first = match symbol {
                Symbol::Non(non_terminal) => non_terminal_firsts.get(&non_terminal).unwrap().clone(),
                Symbol::Terminal(symbol) => First::from_char(symbol)
            };
            result.concatenate(&cur_first);
            if !result.has_epsilon {
                break;
            }
        }
        result
    }

    fn first_of_situation<'a>(non_terminal_firsts: &HashMap<NonTerminal, First>, situation: &GrammarSituation<'a>) -> Self {
        Self::get_first(&non_terminal_firsts, situation.get_rule().right[situation.point..].iter().map(|x| *x))
    }
}

#[derive(Clone, Debug)]
struct LR1AutomatonState<'a> {
    situations: HashMap<GrammarSituation<'a>, First>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FrozenLR1AutomatonState<'a> {
    situations: Vec<(GrammarSituation<'a>, First)>,
}

impl<'a> LR1AutomatonState<'a> {
    fn closure(&mut self, non_terminal_firsts: &HashMap<NonTerminal, First>, grammar: &'a Grammar) {
        let mut unused = self.situations.clone();

        while !unused.is_empty() {
            let cur = unused.iter().next().unwrap();
            let situation = cur.0.clone();
            let first = cur.1.clone();
            unused.remove(&situation);
            let non_terminal = situation.next().and_then(|x| match x {Symbol::Non(non) => Some(non), _ => None});
            match non_terminal {
                Some (non) => {
                    let mut new_first = First::first_of_situation(non_terminal_firsts, &situation.move_point());
                    new_first.concatenate(&first);
                    for new_rule in grammar.get_rules(non) {
                        let new_situation = GrammarSituation::new(new_rule, 0);
                        if !self.situations.contains_key(&new_situation) {
                            self.situations.insert(new_situation.clone(), new_first.clone());
                            unused.insert(new_situation, new_first.clone());
                        } else {
                            let old_first = self.situations.get(&new_situation).unwrap();
                            let mut updated_first = old_first.clone();
                            updated_first.unite(&new_first);
                            if &updated_first != old_first {
                                *self.situations.get_mut(&new_situation).unwrap() = updated_first.clone();
                                unused.remove(&new_situation);
                                unused.insert(new_situation, updated_first);
                            }
                        }
                    }
                }
                _ => ()
            }
        }
    }

    fn freeze(self) -> FrozenLR1AutomatonState<'a> {
        let mut result: Vec<(GrammarSituation<'a>, First)> = self.situations.into_iter().collect();
        result.sort();
        FrozenLR1AutomatonState {
            situations: result
        }
    }
}

impl<'a> FrozenLR1AutomatonState<'a> {
    fn goto_options(&self) -> Vec<Symbol> {
        let mut result: Vec<Symbol> = self.situations.iter().map(|(situation, _)| {
            situation.next()
        }).filter(|x| !x.is_none()).map(|x| x.unwrap()).collect();
        result.sort();
        result.dedup();
        result
    }

    fn goto(&self, non_terminal_firsts: &HashMap<NonTerminal, First>, grammar: &'a Grammar, next: Symbol) -> LR1AutomatonState<'a> {
        let result: HashMap<GrammarSituation, First> = self.situations.iter().filter(|(situation, _)| {
            let next = situation.next().and_then(|x| if x == next {Some(())} else {None});
            next == Some(())
        }).map(|(situation, first)| {
            (situation.clone().move_point(), first.clone())
        }).collect();
        let mut new_state = LR1AutomatonState {
            situations: result
        };
        new_state.closure(non_terminal_firsts, grammar);
        new_state
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LR1Action {
    Shift(usize),
    Reduce(usize),
    Accept
}

#[derive(Clone, Debug)]
pub struct LR1Algorithm {
    action: Vec<HashMap<Option<char>, LR1Action>>,
    goto: Vec<HashMap<NonTerminal, usize>>,
    rules: Vec<GrammarRule>,
}

impl ParsingAlgorithm for LR1Algorithm {
    fn fit(mut grammar: Grammar) -> Option<Self> {
        let starting_rule = grammar.add_exclusive_starting_non_terminal().clone();
        let starting_situation = GrammarSituation::new(&starting_rule, 0);

        let non_terminals = grammar.get_non_terminals();
        let mut non_terminal_firsts: HashMap<NonTerminal, First> = non_terminals.iter().map(|x| (*x, First::new())).collect();

        loop {
            let mut new_firsts = non_terminal_firsts.clone();
            for (non_terminal, _) in non_terminal_firsts.iter() {
                for rule in grammar.get_rules(*non_terminal).iter() {
                    new_firsts.get_mut(non_terminal).unwrap().unite(&First::get_first(&non_terminal_firsts, rule.right.iter().map(|x| *x)));
                }
            }
            if new_firsts != non_terminal_firsts {
                non_terminal_firsts = new_firsts
            } else {
                break;
            }
        }

        let mut rules_list: Vec<GrammarRule> = Vec::new();
        let mut reverse_rules_list: HashMap<GrammarRule, usize> = HashMap::new();
        for non in non_terminals {
            for rule in grammar.get_rules(non) {
                reverse_rules_list.insert(rule.clone(), rules_list.len());
                rules_list.push(rule.clone());
            }
        }
        let accept_index = *reverse_rules_list.get(&starting_rule).unwrap();

        let mut states_list: Vec<FrozenLR1AutomatonState> = Vec::new();

        let mut reverse_states_list: HashMap<FrozenLR1AutomatonState, usize> = HashMap::new();
        let mut action: Vec<HashMap<Option<char>, LR1Action>> = Vec::new();
        let mut goto: Vec<HashMap<NonTerminal, usize>> = Vec::new();
        
        let mut start_situation = HashMap::new();
        let start_first = First::from_empty_string();
        start_situation.insert(starting_situation, start_first);
        let mut start_lr1_state = LR1AutomatonState {
            situations: start_situation
        };
        start_lr1_state.closure(&non_terminal_firsts, &grammar);
        let start_lr1_state = start_lr1_state.freeze();
        states_list.push(start_lr1_state.clone());
        reverse_states_list.insert(start_lr1_state, 0);

        let mut index = 0;
        while index < states_list.len() {
            action.push(HashMap::new());
            goto.push(HashMap::new());
            let cur_state = states_list[index].clone();
            for option in cur_state.goto_options() {
                let new_state = states_list[index].clone().goto(&non_terminal_firsts, &grammar, option).freeze();
                let new_index = if reverse_states_list.contains_key(&new_state) {
                    *reverse_states_list.get(&new_state).unwrap()
                } else {
                    reverse_states_list.insert(new_state.clone(), states_list.len());
                    states_list.push(new_state.clone());
                    states_list.len() - 1
                };
                match option {
                    Symbol::Non(non_terminal) => {
                        goto[index].insert(non_terminal, new_index);
                    }
                    Symbol::Terminal(symbol) => {
                        if !action[index].insert(Some(symbol), LR1Action::Shift(new_index)).is_none() {
                            return None;
                        }
                    }
                }
            }
            for (situation, first) in cur_state.situations.iter() {
                if situation.next().is_none() {
                    let rule_index = *reverse_rules_list.get(situation.get_rule()).unwrap();
                    let cur_action = if rule_index == accept_index {LR1Action::Accept} else {LR1Action::Reduce(rule_index)};
                    for symbol in first.characters.iter() {
                        if !action[index].insert(Some(*symbol), cur_action).is_none() {
                            return None;
                        }
                    }
                    if first.has_epsilon {
                        if !action[index].insert(None, cur_action).is_none() {
                            return None;
                        }
                    }
                }
            }
            index += 1;
        }

        Some(Self {
            action,
            goto,
            rules: rules_list,
        })
    }

    fn predict(&self, word: &String) -> bool {
        #[derive(Clone, Copy, Debug)]
        enum LR1Symbol {
            Non(NonTerminal),
            Terminal(Option<char>)
        }
        #[derive(Clone, Copy, Debug)]
        enum StackContent {
            State(usize),
            Symbol(LR1Symbol)
        }
        use StackContent::*;
        use LR1Action::*;
        use LR1Symbol::*;

        let mut lr1_stack = vec![State(0)];

        let mut pos = 0;

        let mut indexable_word: Vec<Option<char>> = word.chars().map(|x| Some(x)).collect();
        indexable_word.push(None);
        let mut result = || {loop { // made it to use ? operator
            if pos >= indexable_word.len() {
                break None;
            }
            let next = indexable_word[pos];
            let cur_state = match lr1_stack.last().unwrap() {
                Symbol(_) => panic!("invalid LR1 stack"),
                State(x) => *x
            };
            match self.action[cur_state].get(&next)? {
                Shift(next_state) => {
                    lr1_stack.push(Symbol(Terminal(next)));
                    lr1_stack.push(State(*next_state));
                    pos += 1;
                }
                Reduce(rule_index) => {
                    let rule = &self.rules[*rule_index];
                    for _ in 0..2 * rule.right.len() {
                        lr1_stack.pop().unwrap();
                    }
                    let cur_state = match lr1_stack.last().unwrap() {
                        Symbol(_) => panic!("invalid LR1 stack"),
                        State(x) => *x
                    };
                    let next_state = self.goto[cur_state].get(&rule.left)?;
                    lr1_stack.push(Symbol(Non(rule.left)));
                    lr1_stack.push(State(*next_state));
                }
                Accept => break Some(())
            }
        }};
        match result() {
            Some(()) => true,
            None => false
        }
    }
}

#[cfg(test)]
pub mod tests {
    use rand::prelude::*;

    use super::*;
    use crate::grammars::earley::{tests::*, EarleyAlgorithm};
    use Symbol::*;
    use LR1Action::*;

    fn import_ccdcd_table() -> LR1Algorithm {
        // LR1-table for the grammar S->CC, C->cC, C->d taken from https://neerc.ifmo.ru/wiki/index.php?title=LR1(1)-разбор
        let s = NonTerminal::new(0);
        let c = NonTerminal::new(1);
        let rules = vec![
            GrammarRule::new(s, vec![Non(c), Non(c)]),
            GrammarRule::new(c, vec![Terminal('c'), Non(c)]),
            GrammarRule::new(c, vec![Terminal('d')]),
        ];
        let cnt_states = 10usize;
        let mut action: Vec<HashMap<Option<char>, LR1Action>> = (0..cnt_states).map(|_| HashMap::new()).collect();
        let mut goto: Vec<HashMap<NonTerminal, usize>> = (0..cnt_states).map(|_| HashMap::new()).collect();
        goto[0].insert(s, 1);
        goto[0].insert(c, 2);
        goto[2].insert(c, 5);
        goto[3].insert(c, 8);
        goto[6].insert(c, 9);

        action[1].insert(None, Accept);

        action[0].insert(Some('c'), Shift(3));
        action[0].insert(Some('d'), Shift(4));
        action[2].insert(Some('c'), Shift(6));
        action[2].insert(Some('d'), Shift(7));
        action[3].insert(Some('c'), Shift(3));
        action[3].insert(Some('d'), Shift(4));
        action[6].insert(Some('c'), Shift(6));
        action[6].insert(Some('d'), Shift(7));

        action[4].insert(Some('c'), Reduce(2)); // fixed mistake in conspect
        action[4].insert(Some('d'), Reduce(2));
        action[5].insert(None, Reduce(0));
        action[7].insert(None, Reduce(2));
        action[8].insert(Some('c'), Reduce(1));
        action[8].insert(Some('d'), Reduce(1));
        action[9].insert(None, Reduce(1));

        return LR1Algorithm {
            rules,
            action,
            goto,
        }
    }

    fn test_ccdcd_grammar_from(algo: LR1Algorithm) {
        assert!(algo.predict(&"ccdcd".to_string()));
        assert!(algo.predict(&"dd".to_string()));
        assert!(algo.predict(&"dccccccd".to_string()));
        assert!(!algo.predict(&String::new()));
        assert!(!algo.predict(&"ccdcdc".to_string()));
        assert!(!algo.predict(&"ccdcdd".to_string()));
        assert!(!algo.predict(&"ccd".to_string()));
        assert!(!algo.predict(&"d".to_string()));
    }

    #[test]
    fn test_ccdcd_imported_table() {
        test_ccdcd_grammar_from(import_ccdcd_table());
    }

    #[test]
    fn test_ccdcd_built_table() {
        use Symbol::*;
        let mut gr = Grammar::new();
        let s = gr.starting;
        let c = gr.new_non_terminal();
        gr.add_rule(GrammarRule::new(s, vec![Non(c), Non(c)]));
        gr.add_rule(GrammarRule::new(c, vec![Terminal('c'), Non(c)]));
        gr.add_rule(GrammarRule::new(c, vec![Terminal('d')]));

        let algo = LR1Algorithm::fit(gr).unwrap();
        test_ccdcd_grammar_from(algo);
    }

    #[test]
    fn test_first_grammar() {
        generic_test_first_grammar::<LR1Algorithm>(false);
    }

    #[test]
    fn test_second_grammar() {
        generic_test_second_grammar::<LR1Algorithm>(false);
    }

    fn gen_random_word<T: Copy>(alphabet: &Vec<T>, len: usize) -> Vec<T> {
        (0..len).map(|_| alphabet[random::<usize>() % alphabet.len()]).collect()
    }

    fn gen_random_grammar(alphabet: &Vec<char>, non_terminals_count: usize, max_rule_len: usize, rule_count: usize) -> Grammar {
        let mut result = Grammar::new();
        let mut non_terminals = vec![result.starting];
        for _ in 1..non_terminals_count {
            non_terminals.push(result.new_non_terminal());
        }
        let symbols = alphabet.iter().map(|x| Symbol::Terminal(*x))
                .chain(non_terminals.iter().map(|x| Symbol::Non(*x))).collect::<Vec<Symbol>>();
        for _ in 0..rule_count {
            let left = non_terminals[random::<usize>() % non_terminals_count];
            let right = gen_random_word(&symbols, random::<usize>() % max_rule_len + 1);
            result.add_rule(GrammarRule::new(left, right));
        }
        result
    }

    #[test]
    fn test_fuzz() {
        const ITERATIONS: usize = 500;
        const WORDS: usize = 1000;
        const MAX_WORD_LEN: usize = 5;
        const ALPHABET_SIZE: u8 = 3;
        const NON_TERMINALS: usize = 5;
        const MAX_RULE_LEN: usize = 3;
        const RULE_COUNT: usize = 10;

        let alphabet: Vec<char> = (0..ALPHABET_SIZE).map(|x| (x + ('a' as u8)) as char).collect();

        let mut cnt_successes = 0;
        let mut cnt_predicts = 0;

        for _iter in 0..ITERATIONS {
            let grammar = gen_random_grammar(&alphabet, NON_TERMINALS, MAX_RULE_LEN, RULE_COUNT);
            let lr1 = match LR1Algorithm::fit(grammar.clone()) {
                Some(x) => x,
                None => {continue;}
            };
            let earley = EarleyAlgorithm::fit(grammar).unwrap();
            for _word in 0..WORDS {
                let word = gen_random_word(&alphabet, MAX_WORD_LEN).into_iter().collect::<String>();
                let res_earley = earley.predict(&word);
                let res_lr1 = lr1.predict(&word);
                assert!(res_lr1 == res_earley);
                if res_earley {
                    cnt_predicts += 1;
                }
            }
            cnt_successes += 1;
        }
        eprintln!("lr1 builds: {}\npredicts: {}", cnt_successes, cnt_predicts);
        assert!(cnt_successes >= 10);
        assert!(cnt_predicts >= 10);
    }
}
