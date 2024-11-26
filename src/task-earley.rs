mod grammars;

use std::io::{BufRead, BufReader};
use grammars::earley::{EarleyAlgorithm, ParsingAlgorithm};

use crate::grammars::context_free::*;

fn main() {
    let mut reader = BufReader::with_capacity(4096, std::io::stdin());
    let grammar = Grammar::read(&mut reader).unwrap();
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
    let algo = EarleyAlgorithm::fit(grammar).unwrap();
    let queries: usize = read_line().expect("m not found").parse().expect("wrong m format");
    for i in 0..queries {
        let query = read_line().expect(format!("query {} not found", i + 1).as_str());
        println!("{}", if algo.predict(&query) {"Yes"} else {"No"});
    }
}
