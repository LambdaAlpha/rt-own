# Runtime Ownership for Rust

This library implements the concept of dynamic ownership by introducing three core types: `Owner`, `Viewer`, and `Holder`. These types serve distinct roles in managing shared data, enabling flexible and safe ownership control at runtime.

## Type Descriptions

- **`Owner`**:  
  Represents exclusive ownership of shared data. The `Owner` can view, modify, or destroy the shared data. Only one `Owner` may exist at a time for a given piece of data, and it cannot coexist with another `Owner` or `Viewer`. However, it may coexist with one or more `Holder` instances.

- **`Viewer`**:  
  Represents shared, read-only access to the data. A `Viewer` can view the shared data but cannot modify it. Multiple `Viewer` instances can coexist with each other, and they may also exist alongside `Holder` instances.

- **`Holder`**:  
  Holds a reference to shared data without the ability to directly view or modify it. A `Holder` can be downgraded from an `Owner` or `Viewer`, and later upgraded to either a `Viewer` or an `Owner` as needed.

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
