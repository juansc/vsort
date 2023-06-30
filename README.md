# vsort

A Rust library that implements the GNU version sort algorithm. It follows the spec
given [here](https://github.com/coreutils/coreutils/blob/master/doc/sort-version.texi).

## Why vsort?
Other implementations didn't match the GNU spec, and some were missing tests. The goal is to match the core utils
implementation as close as possible. If you notice any discrepancies please open an issue.

## Why not FFI?
FFI probably your best bet if you need absolute parity with GNU version sort. In the case you want their
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

## TODOs
- [ ] Add examples of code usages in each public function doc
- [ ] Add installation instructions
