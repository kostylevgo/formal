use std::collections::HashMap;

use super::context_free::*;
use super::earley::{GrammarSituation, ParsingAlgorithm};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct First {
    has_epsilon: bool,
    characters: Vec<char>,
}

fn merge_vecs<T: Ord + Clone>(a: &Vec<T>, b: &Vec<T>) -> Vec<T> {
    let mut a_index = 0;
    let mut b_index = 0;
    let mut res = Vec::<T>::new();
    while a_index < a.len() && b_index < b.len() {
        if a_index == a.len() || b_index < b.len() && a[a_index] > b[b_index] {
            res.push(b[b_index].clone());
            b_index += 1;
        } else {
            res.push(a[a_index].clone());
            a_index += 1;
        }
    }
    res
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

    fn from_char(ch: char) -> First {
        Self {
            has_epsilon: false,
            characters: vec![ch]
        }
    }

    fn new() -> First {
        Self {
            has_epsilon: false,
            characters: Vec::new(),
        }
    }

    fn get_first(non_terminal_firsts: &HashMap<NonTerminal, First>, string: impl Iterator<Item = Symbol>) -> First {
        let mut res = First::from_empty_string();
        for symbol in string {
            let cur_first = match symbol {
                Symbol::Non(non_terminal) => non_terminal_firsts.get(&non_terminal).unwrap().clone(),
                Symbol::Terminal(ch) => First::from_char(ch)
            };
            res.concatenate(&cur_first);
            if !res.has_epsilon {
                break;
            }
        }
        res
    }

    fn first_of_situation<'a>(non_terminal_firsts: &HashMap<NonTerminal, First>, situation: &GrammarSituation<'a>) -> Self {
        Self::get_first(&non_terminal_firsts, situation.get_rule().right[situation.point..].iter().map(|x| *x))
    }
}

#[derive(Clone, Debug)]
struct LRAutomatonState<'a> {
    situations: HashMap<GrammarSituation<'a>, First>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FrozenLRAutomatonState<'a> {
    situations: Vec<(GrammarSituation<'a>, First)>,
}

impl<'a> LRAutomatonState<'a> {
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

    fn freeze(self) -> FrozenLRAutomatonState<'a> {
        let mut result: Vec<(GrammarSituation<'a>, First)> = self.situations.into_iter().collect();
        result.sort();
        FrozenLRAutomatonState {
            situations: result
        }
    }
}

impl<'a> FrozenLRAutomatonState<'a> {
    fn goto_options(&self) -> Vec<Symbol> {
        let mut res: Vec<Symbol> = self.situations.iter().map(|(situation, _)| {
            situation.next()
        }).filter(|x| !x.is_none()).map(|x| x.unwrap()).collect();
        res.sort();
        res.dedup();
        res
    }

    fn goto(&self, non_terminal_firsts: &HashMap<NonTerminal, First>, grammar: &'a Grammar, next: Symbol) -> LRAutomatonState<'a> {
        let res: HashMap<GrammarSituation, First> = self.situations.iter().filter(|(situation, _)| {
            let next = situation.next().and_then(|x| if x == next {Some(())} else {None});
            next == Some(())
        }).map(|(situation, first)| {
            (situation.clone().move_point(), first.clone())
        }).collect();
        let mut new_state = LRAutomatonState {
            situations: res
        };
        new_state.closure(non_terminal_firsts, grammar);
        new_state
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum LRAction {
    Shift(usize),
    Reduce(usize),
    Accept
}

#[derive(Clone, Debug)]
pub struct LRAlgorithm {
    action: Vec<HashMap<Option<char>, LRAction>>,
    goto: Vec<HashMap<NonTerminal, usize>>,
    rules: Vec<GrammarRule>,
}

// #[derive(Clone, Debug)]
// pub struct LRAutomaton<'a> {
//     table: LRAlgorithm,
//     states: HashMap<FrozenLRAutomatonState<'a>, usize>,
// }
// 
// impl<'a> LRAutomaton<'a> {
//     fn add_state(&'a mut self, non_terminal_firsts: &'a HashMap<NonTerminal, First>, grammar: &'a Grammar, mut state: LRAutomatonState<'a>) -> usize {
//         struct Helper<'a, 'b> {
//             f: &'b dyn FnMut(&'a Helper, LRAutomatonState<'a>) -> usize
//         }
//         let mut f = |helper: &'a Helper, mut state: LRAutomatonState<'a>| {
//             state.closure(non_terminal_firsts, grammar);
//             let frozen = state.freeze();
//             if self.states.contains_key(&frozen) {
//                 return *self.states.get(&frozen).unwrap()
//             }
//             let new_index = self.states.len();
//             self.states.insert(frozen.clone(), new_index);
//             self.table.action.push(HashMap::new());
//             self.table.goto.push(HashMap::new());
//             let options = frozen.goto_options();
//             for option in options.into_iter() {
//                 match option {
//                     Symbol::Non(non_terminal) => {
//                         self.table.goto[new_index].insert(non_terminal, (helper.f)(helper, frozen.goto(&non_terminal_firsts, &grammar, option)));
//                     }
//                     Symbol::Terminal(terminal) => {

//                     }
//                 }
//             }
//             new_index
//         };
//         let helper = Helper::<'a, '_> {
//             f: &f,
//         };
//         (helper.f)(&helper, state)
//     }
// }

impl ParsingAlgorithm for LRAlgorithm {
    fn fit(mut grammar: Grammar) -> Option<Self> {
        let starting_rule = grammar.add_exclusive_starting_non_terminal().clone();
        let starting_situation = GrammarSituation::new(&starting_rule, 0);

        let non_terminals = grammar.get_non_terminals();
        let mut non_terminal_firsts: HashMap<NonTerminal, First> = non_terminals.iter().map(|x| (*x, First::new())).collect();

        loop {
            let mut new_firsts = non_terminal_firsts.clone();
            for (non_terminal, _) in non_terminal_firsts.iter() {
                for rule in grammar.get_rules(*non_terminal).iter() {
                    new_firsts.get_mut(non_terminal).unwrap().concatenate(&First::get_first(&non_terminal_firsts, rule.right.iter().map(|x| *x)));
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

        let mut states_list: Vec<FrozenLRAutomatonState> = Vec::new();

        let mut reverse_states_list: HashMap<FrozenLRAutomatonState, usize> = HashMap::new();
        let mut action: Vec<HashMap<Option<char>, LRAction>> = Vec::new();
        let mut goto: Vec<HashMap<NonTerminal, usize>> = Vec::new();
        
        let mut start_situation = HashMap::new();
        let start_first = First::first_of_situation(&non_terminal_firsts, &starting_situation);
        start_situation.insert(starting_situation, start_first);
        let mut start_lr_state = LRAutomatonState {
            situations: start_situation
        };
        start_lr_state.closure(&non_terminal_firsts, &grammar);
        let start_lr_state = start_lr_state.freeze();
        states_list.push(start_lr_state.clone());
        reverse_states_list.insert(start_lr_state, 0);

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
                    reverse_states_list.insert(new_state.clone(), rules_list.len());
                    states_list.push(new_state.clone());
                    states_list.len() - 1
                };
                match option {
                    Symbol::Non(non_terminal) => {
                        goto[index].insert(non_terminal, new_index);
                    }
                    Symbol::Terminal(ch) => {
                        if !action[index].insert(Some(ch), LRAction::Shift(new_index)).is_none() {
                            return None;
                        }
                    }
                }
            }
            for (situation, first) in cur_state.situations.iter() {
                if situation.next().is_none() {
                    let rule_index = *reverse_rules_list.get(situation.get_rule()).unwrap();
                    let cur_action = if rule_index == accept_index {LRAction::Accept} else {LRAction::Reduce(rule_index)};
                    for ch in first.characters.iter() {
                        if !action[index].insert(Some(*ch), cur_action).is_none() {
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
        enum LRSymbol {
            Non(NonTerminal),
            Terminal(Option<char>)
        }
        #[derive(Clone, Copy, Debug)]
        enum StackContent {
            State(usize),
            Symbol(LRSymbol)
        }
        use StackContent::*;
        use LRAction::*;
        use LRSymbol::*;

        let mut lr_stack = vec![State(0)];

        let mut pos = 0;

        let mut indexable_word: Vec<Option<char>> = word.chars().map(|x| Some(x)).collect();
        indexable_word.push(None);
        let mut result = || {loop { // made it to use ? operator
            if pos >= indexable_word.len() {
                break None;
            }
            let next = indexable_word[pos];
            let cur_state = match lr_stack.last().unwrap() {
                Symbol(_) => panic!("invalid lr stack"),
                State(x) => *x
            };
            match self.action[cur_state].get(&next)? {
                Shift(next_state) => {
                    lr_stack.push(Symbol(Terminal(next)));
                    lr_stack.push(State(*next_state));
                    pos += 1;
                }
                Reduce(rule_index) => {
                    let rule = &self.rules[*rule_index];
                    for _ in 0..2 * rule.right.len() {
                        lr_stack.pop().unwrap();
                    }
                    let cur_state = match lr_stack.last().unwrap() {
                        Symbol(_) => panic!("invalid lr stack"),
                        State(x) => *x
                    };
                    let next_state = self.goto[cur_state].get(&rule.left)?;
                    lr_stack.push(Symbol(Non(rule.left)));
                    lr_stack.push(State(*next_state));
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
    use super::*;
    use crate::grammars::earley::tests::*;
    use Symbol::*;
    use LRAction::*;

    fn import_ccdcd_table() -> LRAlgorithm {
        // LR-table for the grammar S->CC, C->cC, C->d taken from https://neerc.ifmo.ru/wiki/index.php?title=LR(1)-разбор
        let s = NonTerminal::new(0);
        let c = NonTerminal::new(1);
        let rules = vec![
            GrammarRule::new(s, vec![Non(c), Non(c)]),
            GrammarRule::new(c, vec![Terminal('c'), Non(c)]),
            GrammarRule::new(c, vec![Terminal('d')]),
        ];
        let cnt_states = 10usize;
        let mut action: Vec<HashMap<Option<char>, LRAction>> = (0..cnt_states).map(|_| HashMap::new()).collect();
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

        action[4].insert(Some('c'), Reduce(2)); // fixed conspect's mistake
        action[4].insert(Some('d'), Reduce(2));
        action[5].insert(None, Reduce(0));
        action[7].insert(None, Reduce(2));
        action[8].insert(Some('c'), Reduce(1));
        action[8].insert(Some('d'), Reduce(1));
        action[9].insert(None, Reduce(1));

        return LRAlgorithm {
            rules,
            action,
            goto,
        }
    }

    #[test]
    fn test_ccdcd_grammar() {
        let algo = import_ccdcd_table();

        assert!(algo.predict(&"ccdcd".to_string()));
        assert!(algo.predict(&"dd".to_string()));
        assert!(!algo.predict(&"ccdcdc".to_string()));
        assert!(!algo.predict(&"ccdcdd".to_string()));
        assert!(!algo.predict(&"ccd".to_string()));
        assert!(!algo.predict(&"d".to_string()));
    }

    #[test]
    fn test_first_grammar() {
        generic_test_first_grammar::<LRAlgorithm>();
    }

    #[test]
    fn test_second_grammar() {
        generic_test_second_grammar::<LRAlgorithm>();
    }
}
