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
        if longest_path[i] == usize::MAX {
            longest_path_dfs(&graph, &mut longest_path, i);
        }
        if longest_path[i] >= count {
            return Ok(true);
        }
    }
    Ok(false)
}

// ab + c.aba. * .bac. + . + * a 4
// acb..bab.c. * .ab.ba. + . + *a. b 2

fn main() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    let mut input = input.split_whitespace();
    let count: usize = input.next_back().expect("Not enought input").parse().expect("Wrong input format");
    let letter: char = input.next_back().expect("Not enought input").parse().expect("Wrong input format");
    let reverse_polish = input.collect();
    println!("{}", reverse_polish);
    if solve(reverse_polish, letter, count).expect("Error") {
        println!("YES");
    } else {
        println!("NO");
    }
}
