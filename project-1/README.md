# [Project-1]

## Part 1

- `unimplemented!` macro &mdash; indicates unfinished code
- `cargo test --lib` &mdash; run the tests inside the library
- `cargo test --doc` &mdash; run the doc tests in the library
- `cargo test --bins` &mdash; run test for all binaries
- `cargo test --bin x` &mdash; run test just for the `x` bin
- `cargo test --tests` &mdash; run all tests in `tests` folder
- `cargo test --test foo` &mdash; run the `foo` pattern tests in `tests` folder

## Part 2

- `unreachable!` macro &mdash; indicates unreachable code
- `use std::process::exit` &mdash; terminates the current process with the specified exit code
- `env!("CARGO_PKG_NAME")` &mdash; value for `name` from `[package]`-section in Cargo.toml
- `env!("CARGO_PKG_VERSION")` &mdash; value for `version` from `[package]`-section in Cargo.toml 
- `env!("CARGO_PKG_AUTHORS")` &mdash; value for `authors` from `[package]`-section in Cargo.toml
- `env!("CARGO_PKG_DESCRIPTION")` &mdash; value for `description` from `[package]`-section in Cargo.toml

## Part 3

## Part 4

- how to get `Option<String>` 

```rust,no_run
use std::collections::HashMap;

struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).cloned()
    }
}

```

## Part 5

- `cargo doc --open`

## Part 6

```
rustup component add clippy
rustup component add rustfmt

cargo clippy
cargo fmt
```

<!-- links -->
[Project-1]: https://github.com/pingcap/talent-plan/blob/master/rust/projects/project-1/project.md