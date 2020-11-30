# async-select-all

[![Apache-2.0 licensed](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.37+-lightgray.svg)](README.md#rust-version-requirements)
[![Crate](https://img.shields.io/crates/v/async-select-all.svg)](https://crates.io/crates/async-select-all)
[![API](https://docs.rs/async-select-all/badge.svg)](https://docs.rs/async-select-all)

A futures library adapter for selecting over a list of futures.

## Usage

```Rust
use async_select_all::SelectAll;
use futures::executor::block_on;

async fn inc(i: i32) -> i32 {
    i + 1
}

fn main() {
    let futures = vec![inc(10), inc(5)];
    let mut select_all = SelectAll::from(futures);
    let vec = block_on(async {
        let mut vec = Vec::with_capacity(select_all.len());
        while !select_all.is_empty() {
            let val = select_all.select().await;
            vec.push(val)
        }
        vec.sort();
        vec
    });
    assert_eq!(vec, vec![6, 11]);
}
```

## Rust version requirements

`async-select-all` works on rust 1.37 or later.

## License

This project is licensed under the Apache-2.0 license ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `async-select-all` by you, shall be licensed as Apache-2.0, without any additional
terms or conditions.
