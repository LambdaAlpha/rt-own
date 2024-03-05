# Runtime Ownership for Rust

This library implements the concept of dynamic ownership, providing users with three key types: `Owner`, `Sharer`, and `Holder`. These types can play different roles in managing shared data, enabling flexible ownership control.

Type Descriptions

- `Owner`: Holds ownership of shared data. An `Owner` can read, write, or even drop shared data. Note that an `Owner` type can only coexist with some `Holder` instances and cannot coexist with other `Owner` or `Sharer` instances.
- `Sharer`: Shares ownership of shared data. A `Sharer` can read shared data but cannot perform write operations. `Sharer` can coexist with other `Sharer` or `Holder` instances.
- `Holder`: Holds a reference to shared data but does not own it. A `Holder` cannot read or write shared data; its main purpose is to facilitate role conversion between `Owner` and `Sharer`.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
