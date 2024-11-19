mod data_structures;

use data_structures::*;
use deterministic::DeterministicAutomaton;
use graph::Graph;
use single_accepting::SingleAcceptingAutomaton;
use reg_exp::RegularExpression;

fn solve(str: String, letter: char, cnt: usize) -> Result<bool, String> {
    let aut = match RegularExpression::from_reverse_polish(&str) {
        Ok(reg) => {
            DeterministicAutomaton::from(SingleAcceptingAutomaton::from_regular_expression(reg).into_non_deterministic())
        }
        Err(err) => {
            return Err(err);
        }
    };
    Ok(false)
}

fn main() {

}
