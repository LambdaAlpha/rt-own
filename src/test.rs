use std::ops::Deref;
use std::ops::DerefMut;

use crate::Holder;
use crate::Owner;
use crate::OwnerRef;
use crate::State;
use crate::Viewer;
use crate::ViewerRef;

#[test]
fn test_example_owner_viewer_holder() -> Result<(), State> {
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
    Ok(())
}

#[test]
fn test_example_ref() -> Result<(), State> {
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
    let owner_ref2 = OwnerRef::map(owner_ref1, |s| &mut s[6 ..]);
    assert_eq!(&*owner_ref2, "world!");
    // recover full ownership
    let owner = Owner::from(owner_ref2);

    // ViewerRef example with similar projection chain
    let viewer = Viewer::from(owner);
    let viewer_ref0 = ViewerRef::from(viewer);
    let viewer_ref1 = ViewerRef::map(viewer_ref0, |t| &t.0);
    let viewer_ref2 = ViewerRef::map(viewer_ref1, |s| &s[6 ..]);
    assert_eq!(&*viewer_ref2, "world!");
    Ok(())
}

// explicitly drop all variables to make their lifetime clear
#[test]
fn test_holder() -> Result<(), State> {
    let h1 = Holder::new("".to_owned());
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let h2 = Holder::clone(&h1);
    assert_state(Holder::state(&h1), false, 2, 0, false);
    {
        let h3 = Holder::clone(&h1);
        assert_state(Holder::state(&h1), false, 3, 0, false);
        {
            let _ = Holder::clone(&h1);
            assert_state(Holder::state(&h1), false, 3, 0, false);
        }
        assert_state(Holder::state(&h1), false, 3, 0, false);
        drop(h3);
        assert_state(Holder::state(&h1), false, 2, 0, false);
    }
    assert_state(Holder::state(&h1), false, 2, 0, false);
    let h4 = Holder::clone(&h1);
    assert_state(Holder::state(&h1), false, 3, 0, false);
    let h5 = Holder::clone(&h1);
    assert_state(Holder::state(&h1), false, 4, 0, false);
    drop(h4);
    assert_state(Holder::state(&h1), false, 3, 0, false);
    drop(h2);
    assert_state(Holder::state(&h1), false, 2, 0, false);
    drop(h5);
    assert_state(Holder::state(&h1), false, 1, 0, false);
    drop(h1);
    Ok(())
}

// explicitly drop all variables to make their lifetime clear
#[test]
fn test_viewer() -> Result<(), State> {
    let v1 = Viewer::new("".to_owned());
    assert_state(Viewer::state(&v1), false, 0, 1, false);
    let v2 = Viewer::clone(&v1);
    assert_state(Viewer::state(&v1), false, 0, 2, false);
    {
        let v3 = Viewer::clone(&v1);
        assert_state(Viewer::state(&v1), false, 0, 3, false);
        {
            let _ = Viewer::clone(&v1);
            assert_state(Viewer::state(&v1), false, 0, 3, false);
        }
        assert_state(Viewer::state(&v1), false, 0, 3, false);
        drop(v3);
        assert_state(Viewer::state(&v1), false, 0, 2, false);
    }
    assert_state(Viewer::state(&v1), false, 0, 2, false);
    let v4 = Viewer::clone(&v1);
    assert_state(Viewer::state(&v1), false, 0, 3, false);
    let v5 = Viewer::clone(&v1);
    assert_state(Viewer::state(&v1), false, 0, 4, false);
    drop(v4);
    assert_state(Viewer::state(&v1), false, 0, 3, false);
    drop(v2);
    assert_state(Viewer::state(&v1), false, 0, 2, false);
    drop(v5);
    assert_state(Viewer::state(&v1), false, 0, 1, false);
    drop(v1);
    Ok(())
}

#[test]
fn test_owner() -> Result<(), State> {
    let o = Owner::new("".to_owned());
    assert!(Owner::state(&o).is_owned());
    drop(o);
    Ok(())
}

// explicitly drop all variables to make their lifetime clear
#[test]
fn test_holder_viewer_owner() -> Result<(), State> {
    let h1 = Holder::new("".to_owned());
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let v1 = Viewer::try_from(&h1)?; // viewer with holder
    assert_state(Holder::state(&h1), false, 1, 1, false);
    let v2 = Viewer::clone(&v1); // viewer with holder and viewer
    assert_state(Holder::state(&h1), false, 1, 2, false);
    let _ = Holder::clone(&h1); // holder with viewer and holder
    assert_state(Holder::state(&h1), false, 1, 2, false);
    Owner::try_from(&h1).unwrap_err(); // owner with viewer and holder
    assert_state(Holder::state(&h1), false, 1, 2, false);
    let _ = Holder::from(&v1); // holder with viewer and holder
    assert_state(Holder::state(&h1), false, 1, 2, false);
    drop(v1);
    assert_state(Holder::state(&h1), false, 1, 1, false);
    Owner::try_from(&h1).unwrap_err(); // owner with holder and viewer
    assert_state(Holder::state(&h1), false, 1, 1, false);
    drop(v2);
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let o1 = Owner::try_from(&h1)?; // owner with holder
    assert_state(Holder::state(&h1), false, 1, 0, true);
    Owner::try_from(&h1).unwrap_err(); // owner with holder and owner
    assert_state(Holder::state(&h1), false, 1, 0, true);
    Viewer::try_from(&h1).unwrap_err(); // viewer with holder and owner
    assert_state(Holder::state(&h1), false, 1, 0, true);
    let _ = Holder::clone(&h1); // holder with holder and owner
    assert_state(Holder::state(&h1), false, 1, 0, true);
    drop(o1);
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let v3 = Viewer::try_from(&h1)?; // viewer with holder
    assert_state(Holder::state(&h1), false, 1, 1, false);
    drop(v3);
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let o2 = Owner::try_from(&h1)?;
    assert_state(Holder::state(&h1), false, 1, 0, true);
    Owner::drop_data(o2);
    assert_state(Holder::state(&h1), true, 1, 0, false);
    Viewer::try_from(&h1).unwrap_err();
    assert_state(Holder::state(&h1), true, 1, 0, false);
    let _ = Holder::clone(&h1);
    assert_state(Holder::state(&h1), true, 1, 0, false);
    Holder::reinit(&h1, "reinit".to_owned())?;
    assert_state(Holder::state(&h1), false, 1, 0, false);
    drop(h1);
    Ok(())
}

fn assert_state(
    state: State, is_dropped: bool, holder_cnt: usize, viewer_cnt: usize, is_owned: bool,
) {
    assert_eq!(state.is_dropped(), is_dropped);
    assert_eq!(state.holder_count(), holder_cnt);
    assert_eq!(state.viewer_count(), viewer_cnt);
    assert_eq!(state.is_owned(), is_owned);
}

#[test]
fn test_deref() -> Result<(), State> {
    let v1 = Viewer::new("".to_owned());
    let v2 = Viewer::clone(&v1);
    assert_eq!(v1.deref(), "");
    assert_eq!(v2.deref(), "");
    Ok(())
}

#[test]
fn test_deref_mut() -> Result<(), State> {
    let mut o = Owner::new("".to_owned());
    assert_eq!(o.deref(), "");
    {
        let s = o.deref_mut();
        s.push('1');
    }
    assert_eq!(o.deref(), "1");
    Ok(())
}

#[test]
fn test_take() -> Result<(), State> {
    let o = Owner::new("123".to_owned());
    let h = Holder::from(&o);
    let s = Owner::move_data(o);
    assert_eq!(s, "123".to_owned());
    Viewer::try_from(&h).unwrap_err();
    Owner::try_from(&h).unwrap_err();
    Ok(())
}

#[test]
fn test_reinit() -> Result<(), State> {
    let o = Owner::new("123".to_owned());
    let h = Holder::from(&o);
    Owner::drop_data(o);
    Holder::reinit(&h, "321".to_owned())?;
    let v = Viewer::try_from(&h)?;
    assert_eq!(v.deref(), "321");
    Ok(())
}

#[test]
fn test_circular() -> Result<(), State> {
    struct Circular {
        _ref: Option<Box<Holder<Circular>>>,
    }
    let mut o: Owner<Circular> = Owner::new(Circular { _ref: None });
    let h = Holder::from(&o);

    *o = Circular { _ref: Some(Box::new(h)) };

    drop(o);

    Ok(())
}
