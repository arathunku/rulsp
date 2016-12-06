lisp _almost_ based on [lisp](http://www.lwh.jp/lisp)


# Run


```
cargo install
cargo build --release
./target/release/rulsp repl
```


# TODO

- apply
- better error message (file, line number, location in line)
- modules
- use alternative lexer? (nom?, something else?)
- booleans
- strings
- more comp funcs (>, <, <=, >=, ...?)
- floats
- try/catch or maybe more rusty way to handle errors? with Result
- intercop with Rust(?!)
- get rid of nil and have Option<>
- tree analyzer - verify types before execution as much as possible



## Misc commands

```
RUST_LOG=rulsp=trace cargo run 2 > trace.log 2>&1

cargo build --release && \
    perf record -g target/release/rulsp 10000 && \
    perf script | ~/Applications/FlameGraph/stackcollapse-perf.pl | ~/Applications/FlameGraph/flamegraph.pl > flame-counting.svg


rm callgrind.*
cargo build --release && \
    valgrind --tool=callgrind target/release/rulsp

callgrind_annotate callgrind.out.* | grep "[[:space:]]src\/*" | head -n 10
```
