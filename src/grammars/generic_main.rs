use std::io::{BufRead, BufReader};
use crate::grammars::context_free::{Grammar, ParsingAlgorithm};

pub fn generic_main<Algo: ParsingAlgorithm>(inner: impl std::io::Read) -> Result<Vec<bool>, String> {
    let mut reader = BufReader::with_capacity(4096, inner);
    let grammar = match Grammar::read(&mut reader) {
        Ok(grammar) => grammar,
        Err(err) => return Err(err)
    };
    let mut read_line = || -> Option<String> {
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Err(_) => None,
            Ok(_) => {
                buf.truncate(buf.len() - 1);
                Some(buf)
            }
        }
    };
    let algo = Algo::fit(grammar);
    let algo = match algo {
        Some(algo) => algo,
        _ => return Err("could not build algorithm".to_string())
    };
    let queries = read_line().and_then(|x| Some(x.parse::<usize>()));
    let queries = match queries {
        Some(word) => match word {
            Ok(queries) => queries,
            _ => return Err("could not read m".to_string())
        }
        _ => return Err("could not read m".to_string())
    };
    let mut ans: Vec<bool> = Vec::new();
    for i in 0..queries {
        let query = match read_line() {
            Some(word) => word,
            None => return Err(format!("could not read query {}", i + 1))
        };
        ans.push(algo.predict(&query));
    }
    Ok(ans)
}

#[cfg(test)]
mod tests {
    use crate::grammars::context_free::ParsingAlgorithm;
    use crate::grammars::earley::EarleyAlgorithm;
    use crate::grammars::lr1::LR1Algorithm;
    use std::fs::File;

    use super::generic_main;

    fn test_file<Algo: ParsingAlgorithm> (name: String, expected: Vec<bool>) {
        let file = File::open(name).unwrap();
        let result = generic_main::<Algo>(file);
        assert!(result.unwrap() == expected);
    }

    #[test]
    fn test_earley() {
        test_file::<EarleyAlgorithm>("./tests/test-data/an-bn.txt".to_string(), vec![true, true, false, true, false]);
        test_file::<EarleyAlgorithm>("./tests/test-data/bracket-sequence.txt".to_string(), vec![true, false, true, true]);
    }

    #[test]
    fn test_lr1() {
        test_file::<LR1Algorithm>("./tests/test-data/an-bn.txt".to_string(), vec![true, true, false, true, false]);
    }

    #[test]
    #[should_panic]
    fn test_lr1_fail() {
        test_file::<LR1Algorithm>("./tests/test-data/bracket-sequence.txt".to_string(), vec![true, false, true, true]);
    }
}
