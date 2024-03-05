use std::ops::{
    Deref,
    DerefMut,
};

use crate::{
    Holder,
    Owner,
    Sharer,
    State,
};

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
fn test_sharer() -> Result<(), State> {
    let s1 = Sharer::new("".to_owned());
    assert_state(Sharer::state(&s1), false, 0, 1, false);
    let s2 = Sharer::clone(&s1);
    assert_state(Sharer::state(&s1), false, 0, 2, false);
    {
        let s3 = Sharer::clone(&s1);
        assert_state(Sharer::state(&s1), false, 0, 3, false);
        {
            let _ = Sharer::clone(&s1);
            assert_state(Sharer::state(&s1), false, 0, 3, false);
        }
        assert_state(Sharer::state(&s1), false, 0, 3, false);
        drop(s3);
        assert_state(Sharer::state(&s1), false, 0, 2, false);
    }
    assert_state(Sharer::state(&s1), false, 0, 2, false);
    let s4 = Sharer::clone(&s1);
    assert_state(Sharer::state(&s1), false, 0, 3, false);
    let s5 = Sharer::clone(&s1);
    assert_state(Sharer::state(&s1), false, 0, 4, false);
    drop(s4);
    assert_state(Sharer::state(&s1), false, 0, 3, false);
    drop(s2);
    assert_state(Sharer::state(&s1), false, 0, 2, false);
    drop(s5);
    assert_state(Sharer::state(&s1), false, 0, 1, false);
    drop(s1);
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
fn test_mix_ref() -> Result<(), State> {
    let h1 = Holder::new("".to_owned());
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let s1 = Sharer::try_from(&h1)?; // sharer with holder
    assert_state(Holder::state(&h1), false, 1, 1, false);
    let s2 = Sharer::clone(&s1); // sharer with holder and sharer
    assert_state(Holder::state(&h1), false, 1, 2, false);
    let _ = Holder::clone(&h1); // holder with sharer and holder
    assert_state(Holder::state(&h1), false, 1, 2, false);
    Owner::try_from(&h1).unwrap_err(); // owner with sharer and holder
    assert_state(Holder::state(&h1), false, 1, 2, false);
    let _ = Holder::from(&s1); // holder with sharer and holder
    assert_state(Holder::state(&h1), false, 1, 2, false);
    drop(s1);
    assert_state(Holder::state(&h1), false, 1, 1, false);
    Owner::try_from(&h1).unwrap_err(); // owner with holder and sharer
    assert_state(Holder::state(&h1), false, 1, 1, false);
    drop(s2);
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let o1 = Owner::try_from(&h1)?; // owner with holder
    assert_state(Holder::state(&h1), false, 1, 0, true);
    Owner::try_from(&h1).unwrap_err(); // owner with holder and owner
    assert_state(Holder::state(&h1), false, 1, 0, true);
    Sharer::try_from(&h1).unwrap_err(); // sharer with holder and owner
    assert_state(Holder::state(&h1), false, 1, 0, true);
    let _ = Holder::clone(&h1); // holder with holder and owner
    assert_state(Holder::state(&h1), false, 1, 0, true);
    drop(o1);
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let s3 = Sharer::try_from(&h1)?; // sharer with holder
    assert_state(Holder::state(&h1), false, 1, 1, false);
    drop(s3);
    assert_state(Holder::state(&h1), false, 1, 0, false);
    let o2 = Owner::try_from(&h1)?;
    assert_state(Holder::state(&h1), false, 1, 0, true);
    Owner::drop_data(o2);
    assert_state(Holder::state(&h1), true, 1, 0, false);
    Sharer::try_from(&h1).unwrap_err();
    assert_state(Holder::state(&h1), true, 1, 0, false);
    let _ = Holder::clone(&h1);
    assert_state(Holder::state(&h1), true, 1, 0, false);
    Holder::reinit(&h1, "reinit".to_owned())?;
    assert_state(Holder::state(&h1), false, 1, 0, false);
    drop(h1);
    Ok(())
}

fn assert_state(
    ref_state: State,
    is_dropped: bool,
    holder_cnt: usize,
    sharer_cnt: usize,
    is_owned: bool,
) {
    assert_eq!(ref_state.is_dropped(), is_dropped);
    assert_eq!(ref_state.holder_count(), holder_cnt);
    assert_eq!(ref_state.sharer_count(), sharer_cnt);
    assert_eq!(ref_state.is_owned(), is_owned);
}

#[test]
fn test_deref() -> Result<(), State> {
    let s1 = Sharer::new("".to_owned());
    let s2 = Sharer::clone(&s1);
    assert_eq!(s1.deref(), "");
    assert_eq!(s2.deref(), "");
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
    Sharer::try_from(&h).unwrap_err();
    Owner::try_from(&h).unwrap_err();
    Ok(())
}

#[test]
fn test_reinit() -> Result<(), State> {
    let o = Owner::new("123".to_owned());
    let h = Holder::from(&o);
    Owner::drop_data(o);
    Holder::reinit(&h, "321".to_owned())?;
    let r = Sharer::try_from(&h)?;
    assert_eq!(r.deref(), "321");
    Ok(())
}

#[test]
fn test_circular() -> Result<(), State> {
    struct Circular {
        _ref: Option<Box<Holder<Circular>>>,
    }
    let mut o: Owner<Circular> = Owner::new(Circular { _ref: None });
    let h = Holder::from(&o);

    *o = Circular {
        _ref: Some(Box::new(h)),
    };

    drop(o);

    Ok(())
}
