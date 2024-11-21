mod data_structures;

use data_structures::*;
use deterministic::DeterministicAutomaton;
use graph::Graph;
use single_accepting::SingleAcceptingAutomaton;
use reg_exp::RegularExpression;

fn solve(str: String, letter: char, count: usize) -> Result<bool, String> {
    let aut = match RegularExpression::from_reverse_polish(&str) {
        Ok(reg) => {
            DeterministicAutomaton::from(SingleAcceptingAutomaton::from_regular_expression(reg).into_non_deterministic())
        }
        Err(err) => {
            return Err(err);
        }
    };
    let (mut graph, _, is_accepting) = aut.into_fields();
    enum Reachability {
        Reachable,
        Unreachable,
        Unknown
    }
    let mut used: Vec<Reachability> = (0..graph.size()).map(|_| Reachability::Unknown).collect();
    fn accepting_reachable_dfs(graph: &Graph<char>, is_accepting: &Vec<bool>, used: &mut Vec<Reachability>, vertex: usize) {
        used[vertex] = if is_accepting[vertex] {Reachability::Reachable} else {Reachability::Unreachable};
        for edge in graph.get_edges(vertex).iter() {
            match used[edge.to] {
                Reachability::Reachable => {
                    used[vertex] = Reachability::Reachable;
                }
                Reachability::Unknown => {
                    accepting_reachable_dfs(graph, is_accepting, used, edge.to);
                    match used[edge.to] {
                        Reachability::Reachable => {
                            used[vertex] = Reachability::Reachable;
                        }
                        _ => ()
                    }
                }
                _ => ()
            }
        }
    }
    for i in 0..graph.size() {
        match used[i] {
            Reachability::Unknown => {
                accepting_reachable_dfs(&graph, &is_accepting, &mut used, i);
            }
            _ => ()
        }
    }
    graph.retain(|(_from, to, value)| {
        (match used[to] {Reachability::Reachable => true, _ => false}) &&
        *value == letter
    });
    for i in 0..graph.size() {
        for edge in graph.get_edges(i) {
            if edge.to == i {
                return Ok(true);
            }
        }
    }
    let mut color = graph.kosaraju();
    color.sort();
    color.dedup();
    if color.len() < graph.size() {
        return Ok(true);
    }
    let mut longest_path = vec![usize::MAX; graph.size()];
    fn longest_path_dfs(graph: &Graph<char>, longest_path: &mut Vec<usize>, vertex: usize) {
        longest_path[vertex] = 0;
        for edge in graph.get_edges(vertex).iter() {
            if longest_path[edge.to] == usize::MAX {
                longest_path_dfs(graph, longest_path, edge.to);
            }
            longest_path[vertex] = std::cmp::max(longest_path[vertex], longest_path[edge.to] + 1);
        }
    }
    for i in 0..graph.size() {
        match used[i] {
            Reachability::Unreachable => {
                continue;
            }
            _ => ()
        }
        if longest_path[i] == usize::MAX {
            longest_path_dfs(&graph, &mut longest_path, i);
        }
        if longest_path[i] >= count {
            return Ok(true);
        }
    }
    Ok(false)
}

fn main() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    let mut input = input.split_whitespace();
    let count: usize = input.next_back().expect("Not enought input").parse().expect("Wrong input format");
    let letter: char = input.next_back().expect("Not enought input").parse().expect("Wrong input format");
    let reverse_polish = input.collect();
    if solve(reverse_polish, letter, count).expect("Error") {
        println!("YES");
    } else {
        println!("NO");
    }
}

#[cfg(test)]
pub mod tests {
    use crate::reg_exp::RegularExpression;

    use super::solve;

    #[test]
    fn test_first_sample() {
        let polish = String::from("ab+c.aba.*.bac.+.+*");
        assert!(RegularExpression::from_reverse_polish(&polish).unwrap().to_string() == "(((a + b)c) + ((a(ba)*)(b + (ac))))*");
        assert!(solve(polish.clone(), 'a', 3).expect("") == false);
        assert!(solve(polish, 'a', 2).expect("") == true);
    }

    #[test]
    fn test_second_sample() {
        let polish = String::from("acb..bab.c.*.ab.ba.+.+*a.");
        assert!(RegularExpression::from_reverse_polish(&polish).unwrap().to_string() == "(((a(cb)) + ((b((ab)c)*)((ab) + (ba))))*a)");
        assert!(solve(polish.clone(), 'b', 3).expect("") == true);
        assert!(solve(polish, 'b', 4).expect("") == false);
    }

    #[test]
    fn test_zero() {
        assert!(RegularExpression::from_reverse_polish(&String::from("0")).unwrap().to_string() == String::from("0"));
        assert!(solve(String::from("0"), 'a', 0).expect("") == false);
    }

    #[test]
    fn test_one() {
        assert!(RegularExpression::from_reverse_polish(&String::from("1")).unwrap().to_string() == String::from("1"));
        assert!(solve(String::from("1"), 'a', 0).expect("") == true);
        assert!(solve(String::from("1"), 'a', 1).expect("") == false);
    }
}