# utf8-parser

A stateful one-byte-at-a-time UTF-8 parser. This is useful for things like
building characters from bytes pulled from a UART.

[![Repository](https://img.shields.io/badge/github-utf8--parser-/)](https://github.com/Property404/utf8-parser)
[![crates.io](https://img.shields.io/crates/v/utf8-parser.svg)](https://crates.io/crates/utf8-parser)
[![Documentation](https://docs.rs/utf8-parser/badge.svg)](https://docs.rs/utf8-parser)

## Example

```rust
use utf8_parser::Utf8Parser;

let mut parser = Utf8Parser::new();
assert!(parser.push(0xf0).unwrap().is_none());
assert!(parser.push(0x9f).unwrap().is_none());
assert!(parser.push(0x8e).unwrap().is_none());
assert_eq!(parser.push(0x84).unwrap(), Some('🎄'));
```

## Crate Features

* `std` - Enables the
    [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html)
    implementation on `Utf8ParserError`
* `error_in_core` - Same as `std`, but with
    [core::error::Error](https://doc.rust-lang.org/core/error/trait.Error.html).
    Currently requires Nightly and `#![feature(error_in_core)]`. Has no effect
    if the `std` feature is enabled.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](https://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](https://opensource.org/licenses/MIT))

at your option.
