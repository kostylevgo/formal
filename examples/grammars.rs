use formal::grammars::{context_free::{Grammar, GrammarRule, ParsingAlgorithm, Symbol}, earley::EarleyAlgorithm, lr1::LR1Algorithm};

fn ambiguous_bracket_sequences() {
    let mut bracket_sequences = Grammar::new();
    let starting = bracket_sequences.starting;

    bracket_sequences.add_rule(GrammarRule::new(starting, vec![])); // S -> empty string

    bracket_sequences.add_rule(GrammarRule::new(starting,
        vec![Symbol::Terminal('('), Symbol::Non(starting), Symbol::Terminal(')')])); // S -> (S)
    
    bracket_sequences.add_rule(GrammarRule::new(starting,
        vec![Symbol::Terminal('['), Symbol::Non(starting), Symbol::Terminal(']')])); // S -> [S]
    
    bracket_sequences.add_rule(GrammarRule::new(starting,
        vec![Symbol::Terminal('<'), Symbol::Non(starting), Symbol::Terminal('>')])); // S -> <S>
    
    bracket_sequences.add_rule(GrammarRule::new(starting,
        vec![Symbol::Non(starting), Symbol::Non(starting)])); // S -> SS
    
    let parser = EarleyAlgorithm::fit(bracket_sequences.clone()).unwrap();

    println!("[]()<([()]())> is a bracket sequence: {}", parser.predict(&String::from("[]()<([()]())>")));
    println!("[]()<([()]()>) is a bracket sequence: {}", parser.predict(&String::from("[]()<([()]()>)")));

    println!("LR1 parser cannot parse unambiguous grammars: {}", LR1Algorithm::fit(bracket_sequences).is_none());
}

fn unambiguous_bracket_sequences() {
    let mut unambiguous_bracket_sequences = Grammar::new();
    let starting = unambiguous_bracket_sequences.starting;

    unambiguous_bracket_sequences.add_rule(GrammarRule::new(starting, vec![])); // S -> empty string

    unambiguous_bracket_sequences.add_rule(GrammarRule::new(starting,
        vec![Symbol::Terminal('('), Symbol::Non(starting), Symbol::Terminal(')'), Symbol::Non(starting)])); // S -> (S)S
    
    unambiguous_bracket_sequences.add_rule(GrammarRule::new(starting,
        vec![Symbol::Terminal('['), Symbol::Non(starting), Symbol::Terminal(']'), Symbol::Non(starting)])); // S -> [S]S
    
    unambiguous_bracket_sequences.add_rule(GrammarRule::new(starting,
        vec![Symbol::Terminal('<'), Symbol::Non(starting), Symbol::Terminal('>'), Symbol::Non(starting)])); // S -> <S>S

    let parser = LR1Algorithm::fit(unambiguous_bracket_sequences).unwrap();

    println!("[]()<([()]())> is a bracket sequence: {}", parser.predict(&String::from("[]()<([()]())>")));

    println!("[]()<([()]()>) is a bracket sequence: {}", parser.predict(&String::from("[]()<([()]()>)")));
}

fn main() {
    ambiguous_bracket_sequences();
    unambiguous_bracket_sequences();
}