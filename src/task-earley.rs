mod grammars;
use grammars::earley::EarleyAlgorithm;
use grammars::generic_main::generic_main;

fn main() {
    match generic_main::<EarleyAlgorithm>(std::io::stdin()) {
        Ok(ans) => {
            for line in ans {
                println!("{}", if line {"Yes"} else {"No"});
            }
        }
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}