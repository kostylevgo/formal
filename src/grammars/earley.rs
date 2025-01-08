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

#[derive(Clone, Debug)]
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
            situations_with_j_i: Vec<Vec<Vec<GrammarSituation<'a>>>>,
            reached_situations: Vec<Vec<HashSet<GrammarSituation<'a>>>>,
            queue: Vec<Vec<(GrammarSituation<'a>, usize)>>,
            grammar: &'a Grammar,
            indexable_word: Vec<char>,
            len: usize
        }

        impl<'a> EarleyData<'a> {
            fn add(&mut self, situation: GrammarSituation<'a>, j: usize, i: usize) {
                if self.reached_situations[j][i].insert(situation.clone()) {
                    self.queue[j].push((situation.clone(), i));
                    self.situations_with_j_i[j][i].push(situation);
                }
            }
        }

        let mut algo = EarleyData {
            situations_with_j_i: (0..=len).map(|j| (0..=j).map(|_i| Vec::new()).collect()).collect(),
            reached_situations: (0..=len).map(|j| (0..=j).map(|_i| HashSet::new()).collect()).collect(),
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

        fn complete(algo: &mut EarleyData, left: NonTerminal, j: usize, i: usize) {
            for k in 0..=i {
                let mut parent_index = 0;
                // not using algo.situations_with_j_i.iter() because it borrows algo and doesn't allow algo.add
                // in C++, that also could result in iterator invalidation caused by algo.add
                while parent_index < algo.situations_with_j_i[i][k].len() {
                    let parent_situation = algo.situations_with_j_i[i][k].index(parent_index);
                    parent_index += 1;
                    let next = parent_situation.next();
                    if next.is_none() {
                        continue;
                    }
                    if next.unwrap() == Symbol::Non(left) {
                        let mut new_parent_situation = parent_situation.clone();
                        new_parent_situation.point += 1;
                        algo.add(new_parent_situation, j, k);
                    }
                }
            }
        }

        fn reverse_complete<'a>(algo: &mut EarleyData<'a>, situation: &GrammarSituation<'a>, j: usize, i: usize, next: NonTerminal) {
            let mut parent_index = 0;
            // same reason not to use algo.situations.iter() as in complete
            while parent_index < algo.situations_with_j_i[j][j].len() {
                let child_situation = &algo.situations_with_j_i[j][j][parent_index];
                if child_situation.rule.left == next && child_situation.next().is_none() {
                    let mut new_situation = situation.clone();
                    new_situation.point += 1;
                    algo.add(new_situation, j, i);
                }
                parent_index += 1;
            }
        }

        fn process_situation<'a>(algo: &mut EarleyData<'a>, j: usize, index: usize) {
            let (situation, i) = algo.queue[j][index].clone();
            let next = situation.next();
            match next {
                Some(symbol) => match symbol {
                    Symbol::Non(non_terminal) => {
                        predict(algo, j, non_terminal);
                        reverse_complete(algo, &situation, j, i, non_terminal);
                    }
                    Symbol::Terminal(symbol) => {
                        scan(algo, &situation, j, i, symbol);
                    }
                }
                None => {
                    complete(algo, situation.rule.left, j, i);
                }
            }
        }

        for j in 0..=len {
            let mut index = 0;
            while index < algo.queue[j].len() {
                process_situation(&mut algo, j, index);
                index += 1;
            }
        }

        /*
            Why algorithm is O(len^3), if the grammar's size is constant:
            1) for every j and i, only O(1) situations can be added in reached_situations[i][j]
            2) => O(len^2) situations can be added in reached_situations in total, same in queue
            3) every situation in queue is processed only once
            4) processing of a situation is a predict+reverse_complete, scan or complete:
                - predict is O(1)
                - scan is O(1)
                - reverse_complete: one iteration over situations_with_j_i[j][j]
                - complete: O(len) iterations over situations_with_j_i[j][i]
                - for every j and i, situations_with_j_i[j][i] only has O(1) situations, same as in reached_situations[j][i]
                => process_situation is O(len)
            => everything is O(len^3)
         */

        return algo.reached_situations[len][0].contains(&GrammarSituation::new(&self.starting_rule, 1))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use Symbol::*;

    pub fn make_bracket_sequence_or_palindrome_grammar() -> Grammar {
        let mut result = Grammar::new();
        let start = result.starting;
        let start_bracket = result.new_non_terminal();
        let start_palindrome = result.new_non_terminal();
        result.add_rule(GrammarRule::new(start, vec![Non(start_bracket)]));
        result.add_rule(GrammarRule::new(start, vec![Non(start_palindrome)]));

        result.add_rule(GrammarRule::new(start_bracket, vec![Terminal('('), Non(start_bracket), Terminal(')'), Non(start_bracket)]));
        result.add_rule(GrammarRule::new(start_bracket, vec![Terminal('['), Non(start_bracket), Terminal(']'), Non(start_bracket)]));
        result.add_rule(GrammarRule::new(start_bracket, Vec::new()));

        for letter in 0..26 {
            let letter = ('a' as u8 + letter as u8) as char;
            result.add_rule(GrammarRule::new(start_palindrome, vec![Terminal(letter), Non(start_palindrome), Terminal(letter)]));
            result.add_rule(GrammarRule::new(start_palindrome, vec![Terminal(letter)]));
        }
        result.add_rule(GrammarRule::new(start_palindrome, Vec::new()));

        result
    }

    pub fn make_arithmetic_grammar_concatenated_with_an_b2nminus1() -> Grammar {
        let mut result = Grammar::new();
        let start = result.starting;
        let start_arith = result.new_non_terminal();
        let start_an_b2nm1 = result.new_non_terminal();
        let start_an_b2n = result.new_non_terminal();
        result.add_rule(GrammarRule::new(start, vec![Non(start_arith), Terminal('|'), Non(start_an_b2nm1)]));

        result.add_rule(GrammarRule::new(start_an_b2nm1, vec![Terminal('a'), Non(start_an_b2n), Terminal('b')]));
        result.add_rule(GrammarRule::new(start_an_b2n, vec![Terminal('a'), Non(start_an_b2n), Terminal('b'), Terminal('b')]));
        result.add_rule(GrammarRule::new(start_an_b2n, Vec::new()));

        let s_sign_spaces = result.new_non_terminal();
        let s_sign = result.new_non_terminal();
        result.add_rule(GrammarRule::new(start_arith, vec![Terminal('0')]));
        result.add_rule(GrammarRule::new(start_arith, vec![Terminal('1')]));
        result.add_rule(GrammarRule::new(start_arith, vec![Non(start_arith), Non(s_sign_spaces), Non(start_arith)]));
        result.add_rule(GrammarRule::new(start_arith, vec![Terminal('('), Non(start_arith), Terminal(')')]));

        result.add_rule(GrammarRule::new(s_sign_spaces, vec![Terminal(' '), Non(s_sign), Terminal(' ')]));

        result.add_rule(GrammarRule::new(s_sign, vec![Terminal('+')]));
        result.add_rule(GrammarRule::new(s_sign, vec![Terminal('-')]));
        result.add_rule(GrammarRule::new(s_sign, vec![Terminal('*')]));
        result.add_rule(GrammarRule::new(s_sign, vec![Terminal('/')]));

        result
    }

    pub fn generic_test_first_grammar<Algo: ParsingAlgorithm>(should_build: bool) {
        let grammar = make_bracket_sequence_or_palindrome_grammar();
        let algo = match Algo::fit(grammar) {
            None => {
                assert!(!should_build);
                return;
            }
            Some(algo) => {
                assert!(should_build);
                algo
            }
        };
        
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

    pub fn generic_test_second_grammar<Algo: ParsingAlgorithm>(should_build: bool) {
        let grammar = make_arithmetic_grammar_concatenated_with_an_b2nminus1();
        let algo = match Algo::fit(grammar) {
            None => {
                assert!(!should_build);
                return;
            }
            Some(algo) => {
                assert!(should_build);
                algo
            }
        };
        
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
        generic_test_first_grammar::<EarleyAlgorithm>(true);
    }

    #[test]
    fn test_second_grammar() {
        generic_test_second_grammar::<EarleyAlgorithm>(true);
    }
}