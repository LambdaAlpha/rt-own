# Runtime Ownership for Rust

This library implements the concept of dynamic ownership by introducing three core types: `Owner`, `Viewer`, and `Holder`. These types serve distinct roles in managing shared data, enabling flexible and safe ownership control at runtime.

## Type Descriptions

- **`Owner`**:  
  Represents exclusive ownership of shared data. The `Owner` can view, modify, or destroy the shared data. Only one `Owner` may exist at a time for a given piece of data, and it cannot coexist with another `Owner` or `Viewer`. However, it may coexist with one or more `Holder` instances.

- **`Viewer`**:  
  Represents shared, read-only access to the data. A `Viewer` can view the shared data but cannot modify it. Multiple `Viewer` instances can coexist with each other, and they may also exist alongside `Holder` instances.

- **`Holder`**:  
  Holds a reference to shared data without the ability to directly view or modify it. A `Holder` can be downgraded from an `Owner` or `Viewer`, and later upgraded to either a `Viewer` or an `Owner` as needed.

## Example

```rust
use rt_own::Holder;
use rt_own::Owner;
use rt_own::Viewer;

fn main() {
    // new owner
    let mut owner = Owner::new("hello".to_owned());
    // owner can mutate data
    owner.push_str(" world!");
    // owner can view data
    println!("{}", &**owner); // hello world!
    // owner -> viewer
    let viewer1 = Viewer::from(owner);
    // viewer can view data
    println!("{}", &**viewer1); // hello world!
    // multiple `Viewer` instances can coexist with each other
    let viewer2 = Viewer::clone(&viewer1);
    println!("{}", &**viewer2); // hello world!
    // viewer -> holder, viewers may also exist alongside `Holder` instances
    let holder = Holder::from(viewer1);
    // viewer -> owner, this works because viewer2 is the only viewer instance, and owners may coexist with holders
    let owner = Owner::try_from(viewer2).unwrap();
    // owner can drop data, even when there are holders
    Owner::drop_data(owner);
    // holder can reinit data, when data is dropped
    Holder::reinit(&holder, "hello new world!".to_owned()).unwrap();
    // holder -> owner, this works because there is no viewer or owner instance
    let owner = Owner::try_from(holder).unwrap();
    println!("{}", &**owner); // hello new world!
}
```

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
