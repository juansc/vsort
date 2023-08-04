[![TESTS](https://github.com/juansc/vsort/actions/workflows/rust.yml/badge.svg)](https://github.com/juansc/vsort/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Latest version](https://img.shields.io/crates/v/vsort.svg)](https://crates.io/crates/vsort)

# vsort

A Rust library that implements the GNU version sort algorithm. It follows the spec
given [here](https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi).

## Installation
```shell
cargo add vsort
```

## Why vsort?
Other version sort implementations don't match the GNU spec, and some were missing tests. The goal is to match the 
behavior of the core utils implementation as close as possible. If you notice any discrepancies please open an issue.

## Why not FFI?
FFI is probably your best bet if you need absolute parity with GNU version sort. In the case you want their
algorithm in Rust here it is :) 

## Usage:

```rust
use vsort::{compare, sort};

fn main() {
    let mut file_names = vec![
        "a.txt",
        "b 1.txt",
        "b 10.txt",
        "b 11.txt",
        "b 5.txt",
        "Ssm.txt",
    ];

    // Pass to sort_by
    file_names.sort_by(|a, b| compare(a, b));
    assert_eq!(
        file_names,
        vec!["Ssm.txt", "a.txt", "b 1.txt", "b 5.txt", "b 10.txt", "b 11.txt"]
    );

    let mut file_names = vec![
        "a.txt",
        "b 1.txt",
        "b 10.txt",
        "b 11.txt",
        "b 5.txt",
        "Ssm.txt",
    ];
    // Alternatively
    sort(&mut file_names);
    assert_eq!(
        file_names,
        vec!["Ssm.txt", "a.txt", "b 1.txt", "b 5.txt", "b 10.txt", "b 11.txt"]
    );
}
```
