# Runtime Ownership for Rust

This library provides dynamic ownership management with runtime enforcement of ownership rules. The type system ensures safe data access patterns while maintaining flexibility.

## Core Types

### Primary Ownership

- **`Owner<T>`** - Exclusive ownership with modify and view rights
- **`Viewer<T>`** - Shared read-only view access
- **`Holder<T>`** - Opaque reference that can be upgraded to `Owner<T>` or `Viewer<T>`

### Projection Types

- **`OwnerRef<S, T>`** - Exclusive ownership with field projection
- **`ViewerRef<S, T>`** - Read-only view with field projection

## Ownership Rules

- **Exclusive Access**: `Owner`/`OwnerRef` cannot coexist with other `Owner`, `OwnerRef`, or `Viewer`/`ViewerRef`
- **Shared View**: Multiple `Viewer`/`ViewerRef` instances can coexist
- **Reference Holding**: All types may coexist with `Holder` instances

## Type Conversions

### Creation

- `Owner<T>` → `OwnerRef<T, T>`
- `Viewer<T>` → `ViewerRef<T, T>`
- Any type can be downgraded to `Holder<T>`

### Recovery

- `OwnerRef<S, T>` → `Owner<S>`
- `ViewerRef<S, T>` → `Viewer<S>`
- `Holder<T>` can upgrade to `Owner<T>` or `Viewer<T>`

## Projection & Mapping

The `*Ref` types enable flexible field access:

- **Projection**: View nested fields of the root object to arbitrary depth
- **Mapping**: `*Ref<A, B>` → `*Ref<A, C>` where `B` is a direct or indirect field of `A` and `C` is a direct or indirect field of `B`
- **Preservation**: All operations maintain the original ownership semantics

## Example

Example for `Owner`, `Viewer` and `Holder`:

```rust
use rt_own::Holder;
use rt_own::Owner;
use rt_own::Viewer;

fn main() {
    // new Owner
    let mut owner = Owner::new("hello".to_owned());
    // Owner can mutate data
    owner.push_str(" world!");
    // Owner can view data
    assert_eq!(&**owner, "hello world!");
    // Owner -> viewer
    let viewer1 = Viewer::from(owner);
    // Viewer can view data
    assert_eq!(&**viewer1, "hello world!");
    // multiple Viewer instances can coexist with each other
    let viewer2 = Viewer::clone(&viewer1);
    assert_eq!(&**viewer2, "hello world!");
    // Viewer -> Holder, Viewers may also exist alongside `Holder` instances
    let holder = Holder::from(viewer1);
    // Viewer -> Owner, this works because viewer2 is the only Viewer instance,
    // and Owners may coexist with Holders
    let owner = Owner::try_from(viewer2).unwrap();
    // Owner can drop data, even when there are Holders
    Owner::drop_data(owner);
    // Holder can reinit data, when data is dropped
    Holder::reinit(&holder, "hello new world!".to_owned()).unwrap();
    // Holder -> Owner, this works because there is no Viewer or Owner instance
    let owner = Owner::try_from(holder).unwrap();
    assert_eq!(&**owner, "hello new world!");
}
```

Example for `OwnerRef` and `ViewerRef`:

```rust
use rt_own::Owner;
use rt_own::OwnerRef;
use rt_own::Viewer;
use rt_own::ViewerRef;

fn main() {
    let owner = Owner::new(("hello".to_owned(), 1));
    // Owner -> OwnerRef for field projection
    let mut owner_ref0 = OwnerRef::from(owner);
    // modify both fields through OwnerRef
    owner_ref0.0.push_str(" world");
    assert_eq!(owner_ref0.0, "hello world");
    owner_ref0.1 = 2;
    assert_eq!(owner_ref0.1, 2);
    // project to string field for focused mutable access
    let mut owner_ref1 = OwnerRef::map(owner_ref0, |t| &mut t.0);
    owner_ref1.push('!');
    assert_eq!(&*owner_ref1, "hello world!");
    // further project to substring
    let owner_ref2 = OwnerRef::map(owner_ref1, |s| &mut s[6..]);
    assert_eq!(&*owner_ref2, "world!");
    // recover full ownership
    let owner = Owner::from(owner_ref2);
  
    // ViewerRef example with similar projection chain
    let viewer = Viewer::from(owner);
    let viewer_ref0 = ViewerRef::from(viewer);
    let viewer_ref1 = ViewerRef::map(viewer_ref0, |t| &t.0);
    let viewer_ref2 = ViewerRef::map(viewer_ref1, |s| &s[6..]);
    assert_eq!(&*viewer_ref2, "world!");
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
