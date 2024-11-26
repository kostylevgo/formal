mod grammars;
use grammars::lr1::LR1Algorithm;
use grammars::generic_main::generic_main;

fn main() {
    match generic_main::<LR1Algorithm>(std::io::stdin()) {
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
