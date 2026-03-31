use formal::data_structures::reg_exp::RegularExpression;

fn main() {
    let abc_strings = RegularExpression::from("ab+c+*"); // reverse polish notation for [a-c]*
    let automaton = abc_strings.unwrap().compile();
    println!("abcccaabac consists only of a, b, c: {}", automaton.check("abcccaabac"));
    println!("abccbfaab consists only of a, b, c: {}", automaton.check("abccbfaab"));

    println!("");

    let anti_greedy_parsing = RegularExpression::from("ab+*b.") // reverse polish notation for [ab]*b
        .unwrap().compile();
    println!("parsing algorithms use automatons: {}", anti_greedy_parsing.check("abbbababab"));

    println!("");

    println!("{:?}", RegularExpression::from("01.").unwrap()); // note that 0 and 1 are reserved for {} and {""} regular expressions
    println!("{}", RegularExpression::from("0").unwrap().compile().check(""));
    println!("{}", RegularExpression::from("1").unwrap().compile().check(""));
}
