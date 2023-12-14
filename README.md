# utf8-parser

A stateful one-byte-at-a-time UTF-8 parser. This is useful for things like
building characters from bytes pulled from a UART.

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
    [Error](https://doc.rust-lang.org/beta/core/error/trait.Error.html)
    implementation on `Utf8ParserError`

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](https://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](https://opensource.org/licenses/MIT))

at your option.
