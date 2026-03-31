Library for parsing formal languages (regular expressions, context-free grammars)

## Installing
```
cargo update
cargo build
cargo install cargo-llvm-cov       # for test coverage report
```

## Examples
```
cargo run --bin example-automatons # ./examples/automatons.rs
cargo run --bin example-grammars   # ./examples/grammars.rs
``` 

## Coverage report
```
cargo llvm-cov --html
```
