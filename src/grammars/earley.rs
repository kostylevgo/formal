use std::collections::HashSet;

use std::ops::Index;

use super::context_free::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GrammarSituation<'a> {
    rule: &'a GrammarRule,
    pub point: usize
}

impl<'a> GrammarSituation<'a> {
    pub fn new(rule: &'a GrammarRule, point: usize) -> Self {
        Self {
            rule,
            point
        }
    }

    pub fn next(&self) -> Option<Symbol> {
        if self.point == self.rule.right.len() {
            None
        } else {
            Some(self.rule.right[self.point])
        }
    }

    pub fn get_rule(&self) -> &GrammarRule {
        self.rule
    }

    pub fn move_point(self) -> Self {
        Self {
            rule: self.rule,
            point: self.point + if self.point == self.rule.right.len() {0} else {1}
        }
    }
}

// pub trait DerivedFromGrammarSituation {
//     fn get_field(&self) -> &GrammarSituation;

//     fn next(&self) -> Option<Symbol> {
//         let situation = self.get_field();
//         if situation.point == situation.rule.right.len() {
//             None
//         } else {
//             Some(situation.rule.right[situation.point])
//         }
//     }
// }

// impl<'a> DerivedFromGrammarSituation for GrammarSituation<'a> {
//     fn get_field(&self) -> &GrammarSituation {
//         self
//     }
// }

pub trait ParsingAlgorithm : Sized {
    fn fit(grammar: Grammar) -> Option<Self>;
    fn predict(&self, word: &String) -> bool;
}

pub struct EarleyAlgorithm {
    starting_rule: GrammarRule,
    grammar: Grammar
}

impl ParsingAlgorithm for EarleyAlgorithm {
    fn fit(mut grammar: Grammar) -> Option<Self> {
        let starting_rule = grammar.add_exclusive_starting_non_terminal();
        Some(Self {
            starting_rule: starting_rule.clone(),
            grammar
        })
    }

    fn predict(&self, word: &String) -> bool {
        let indexable_word: Vec<char> = word.chars().collect();
        let len = indexable_word.len();

        struct EarleyData<'a> {
            d: Vec<Vec<Vec<GrammarSituation<'a>>>>,
            used: Vec<Vec<HashSet<GrammarSituation<'a>>>>,
            queue: Vec<Vec<(GrammarSituation<'a>, usize)>>,
            grammar: &'a Grammar,
            indexable_word: Vec<char>,
            len: usize
        }

        impl<'a> EarleyData<'a> {
            fn add(&mut self, situation: GrammarSituation<'a>, j: usize, i: usize) {
                if self.used[j][i].insert(situation.clone()) {
                    self.queue[j].push((situation.clone(), i));
                    self.d[j][i].push(situation);
                }
            }
        }

        let mut algo = EarleyData {
            d: (0..=len).map(|j| (0..=j).map(|_i| Vec::new()).collect()).collect(),
            used: (0..=len).map(|j| (0..=j).map(|_i| HashSet::new()).collect()).collect(),
            queue: (0..=len).map(|_j| Vec::new()).collect(),
            grammar: &self.grammar,
            indexable_word,
            len,
        };

        algo.add(GrammarSituation::new(&self.starting_rule, 0), 0, 0);

        fn scan<'a>(algo: &mut EarleyData<'a>, situation: &GrammarSituation<'a>, j: usize, i: usize, next: char) {
            if j < algo.indexable_word.len() && algo.indexable_word[j] == next {
                let mut new_situation = situation.clone();
                new_situation.point += 1;
                algo.add(new_situation, j + 1, i);
            }
        }

        fn predict(algo: &mut EarleyData, j: usize, next: NonTerminal) {
            for rule in algo.grammar.get_rules(next) {
                algo.add(GrammarSituation::new(&rule, 0), j, j);
            }
        }

        fn reverse_complete<'a>(algo: &mut EarleyData<'a>, situation: &GrammarSituation<'a>, j: usize, i: usize, next: NonTerminal) {
            let mut parent_ind = 0;
            while parent_ind < algo.d[j][j].len() {
                let child_situation = &algo.d[j][j][parent_ind];
                if child_situation.rule.left == next {
                    match child_situation.next() {
                        None => {
                            let mut new_situation = situation.clone();
                            new_situation.point += 1;
                            algo.add(new_situation, j, i);
                        }
                        _ => ()
                    }
                }
                parent_ind += 1;
            }
        }

        fn complete(algo: &mut EarleyData, left: NonTerminal, j: usize, i: usize) {
            for k in 0..=i {
                let mut parent_ind = 0;
                while parent_ind < algo.d[i][k].len() {
                    let parent_situation = algo.d[i][k].index(parent_ind);
                    match parent_situation.next() {
                        Some(val) => match val {
                            Symbol::Non(parent_non_terminal) => {
                                if parent_non_terminal == left {
                                    let mut new_parent_situation = parent_situation.clone();
                                    new_parent_situation.point += 1;
                                    algo.add(new_parent_situation, j, k);
                                }
                            }
                            _ => ()
                        }
                        _ => ()
                    }
                    parent_ind += 1;
                }
            }
        }

        fn process_situation<'a>(algo: &mut EarleyData<'a>, j: usize, ind: usize) {
            let (situation, i) = algo.queue[j][ind].clone();
            let next = situation.next();
            match next {
                Some(symbol) => match symbol {
                    Symbol::Non(non_terminal) => {
                        predict(algo, j, non_terminal);
                        reverse_complete(algo, &situation, j, i, non_terminal);
                    }
                    Symbol::Terminal(ch) => {
                        scan(algo, &situation, j, i, ch);
                    }
                }
                None => {
                    complete(algo, situation.rule.left, j, i);
                }
            }
        }

        for j in 0..=len {
            let mut ind = 0;
            while ind < algo.queue[j].len() {
                process_situation(&mut algo, j, ind);
                ind += 1;
            }
        }

        return algo.used[len][0].contains(&GrammarSituation::new(&self.starting_rule, 1))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use Symbol::*;

    pub fn make_bracket_sequence_or_palindrome_grammar() -> Grammar {
        let mut res = Grammar::new();
        let s = res.starting;
        let s_bracket = res.new_non_terminal();
        let s_palindrome = res.new_non_terminal();
        res.add_rule(GrammarRule::new(s, vec![Non(s_bracket)]));
        res.add_rule(GrammarRule::new(s, vec![Non(s_palindrome)]));

        res.add_rule(GrammarRule::new(s_bracket, vec![Terminal('('), Non(s_bracket), Terminal(')'), Non(s_bracket)]));
        res.add_rule(GrammarRule::new(s_bracket, vec![Terminal('['), Non(s_bracket), Terminal(']'), Non(s_bracket)]));
        res.add_rule(GrammarRule::new(s_bracket, Vec::new()));

        for letter in 0..26 {
            let letter = ('a' as u8 + letter as u8) as char;
            res.add_rule(GrammarRule::new(s_palindrome, vec![Terminal(letter), Non(s_palindrome), Terminal(letter)]));
            res.add_rule(GrammarRule::new(s_palindrome, vec![Terminal(letter)]));
        }
        res.add_rule(GrammarRule::new(s_palindrome, Vec::new()));

        res
    }

    pub fn make_arithmetic_grammar_concatenated_with_an_b2nminus1() -> Grammar {
        let mut res = Grammar::new();
        let s = res.starting;
        let s_arith = res.new_non_terminal();
        let s_an_bm = res.new_non_terminal();
        let s_an_b2n = res.new_non_terminal();
        res.add_rule(GrammarRule::new(s, vec![Non(s_arith), Terminal('|'), Non(s_an_bm)]));

        res.add_rule(GrammarRule::new(s_an_bm, vec![Terminal('a'), Non(s_an_b2n), Terminal('b')]));
        res.add_rule(GrammarRule::new(s_an_b2n, vec![Terminal('a'), Non(s_an_b2n), Terminal('b'), Terminal('b')]));
        res.add_rule(GrammarRule::new(s_an_b2n, Vec::new()));

        let s_sign_spaces = res.new_non_terminal();
        let s_sign = res.new_non_terminal();
        res.add_rule(GrammarRule::new(s_arith, vec![Terminal('0')]));
        res.add_rule(GrammarRule::new(s_arith, vec![Terminal('1')]));
        res.add_rule(GrammarRule::new(s_arith, vec![Non(s_arith), Non(s_sign_spaces), Non(s_arith)]));
        res.add_rule(GrammarRule::new(s_arith, vec![Terminal('('), Non(s_arith), Terminal(')')]));

        res.add_rule(GrammarRule::new(s_sign_spaces, vec![Terminal(' '), Non(s_sign), Terminal(' ')]));

        res.add_rule(GrammarRule::new(s_sign, vec![Terminal('+')]));
        res.add_rule(GrammarRule::new(s_sign, vec![Terminal('-')]));
        res.add_rule(GrammarRule::new(s_sign, vec![Terminal('*')]));
        res.add_rule(GrammarRule::new(s_sign, vec![Terminal('/')]));

        res
    }

    pub fn generic_test_first_grammar<Algo: ParsingAlgorithm>() {
        let grammar = make_bracket_sequence_or_palindrome_grammar();
        let algo = Algo::fit(grammar).unwrap();
        
        assert!(algo.predict(&"".to_string()));
        assert!(algo.predict(&"e".to_string()));
        assert!(algo.predict(&"racecar".to_string()));
        assert!(!algo.predict(&"cockeyoungerkasai".to_string()));
        assert!(algo.predict(&"eertree".to_string()));
        assert!(algo.predict(&"earleyyelrae".to_string()));
        assert!(!algo.predict(&"earlyyelrae".to_string()));
        assert!(algo.predict(&"()".to_string()));
        assert!(!algo.predict(&"()b".to_string()));
        assert!(algo.predict(&"([]()[])".to_string()));
        assert!(algo.predict(&"()([()])[]()[()[]]".to_string()));
        assert!(!algo.predict(&"()([()))[]()[()[]]".to_string()));
        assert!(!algo.predict(&"()([(]))[]()[()[]]".to_string()));
    }

    pub fn generic_test_second_grammar<Algo: ParsingAlgorithm>() {
        let grammar = make_arithmetic_grammar_concatenated_with_an_b2nminus1();
        let algo = Algo::fit(grammar).unwrap();
        
        assert!(algo.predict(&"0|ab".to_string()));
        assert!(algo.predict(&"0 + 1|aabbb".to_string()));
        assert!(algo.predict(&"1 / 0|ab".to_string()));
        assert!(algo.predict(&"(1 - 1) / ((0 / 0) + 0 * (1 + 1))|aaaaaaaabbbbbbbbbbbbbbb".to_string()));
        assert!(algo.predict(&"((((((((0))))))))|aaabbbbb".to_string()));

        assert!(!algo.predict(&"|".to_string()));
        assert!(!algo.predict(&"1 - 1/ab".to_string()));
        assert!(!algo.predict(&"(1 - 1) / ((0 / 0) + 0 * (1 + 1))|aaaaaaaabbbbbbbbbbbbbb".to_string()));
        assert!(!algo.predict(&"(1 - 1) / ((0 / 0) + 0 * (1 + 1))|aaaaaaaaabbbbbbbbbbbbbbb".to_string()));
        assert!(!algo.predict(&"(1 - 1) / ((0 / 0) + 0 * (1 + 1))|aaaaaaaabbbbbbbbbbbbbb".to_string()));
        assert!(!algo.predict(&"((0))(1)|aabbb".to_string()));
        assert!(!algo.predict(&"aabbb|0".to_string()));
    }

    #[test]
    fn test_first_grammar() {
        generic_test_first_grammar::<EarleyAlgorithm>();
    }

    #[test]
    fn test_second_grammar() {
        generic_test_second_grammar::<EarleyAlgorithm>();
    }
}