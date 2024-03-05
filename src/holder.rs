use std::{
    fmt::{
        Debug,
        Formatter,
    },
    hash::{
        Hash,
        Hasher,
    },
};

use crate::{
    raw::{
        RawRef,
        RefType,
    },
    Owner,
    Sharer,
    State,
};

pub struct Holder<D: ?Sized> {
    pub(crate) raw: RawRef<D>,
}

impl<D: ?Sized> Holder<D> {
    pub fn new(d: D) -> Self
    where
        D: Sized,
    {
        Holder {
            raw: RawRef::new(d, RefType::Holder),
        }
    }

    pub fn state(h: &Holder<D>) -> State {
        h.raw.shared().state()
    }

    pub fn reinit(h: &Holder<D>, d: D) -> Result<(), State>
    where
        D: Sized,
    {
        let state = h.raw.shared().state();
        if state.is_dropped() {
            // SAFETY: data is dropped
            unsafe {
                h.raw.shared().reinit_data(d);
            }
            Ok(())
        } else {
            Err(state)
        }
    }
}

impl<D: ?Sized> Clone for Holder<D> {
    fn clone(&self) -> Self {
        Holder {
            raw: RawRef::clone_to(&self.raw, RefType::Holder).unwrap(),
        }
    }
}

impl<D: ?Sized> From<&Sharer<D>> for Holder<D> {
    fn from(value: &Sharer<D>) -> Self {
        Holder {
            raw: RawRef::clone_to(&value.raw, RefType::Holder).unwrap(),
        }
    }
}

impl<D: ?Sized> From<Sharer<D>> for Holder<D> {
    fn from(value: Sharer<D>) -> Self {
        Holder {
            raw: RawRef::clone_to(&value.raw, RefType::Holder).unwrap(),
        }
    }
}

impl<D: ?Sized> From<&Owner<D>> for Holder<D> {
    fn from(value: &Owner<D>) -> Self {
        Holder {
            raw: RawRef::clone_to(&value.raw, RefType::Holder).unwrap(),
        }
    }
}

impl<D: ?Sized> From<Owner<D>> for Holder<D> {
    fn from(value: Owner<D>) -> Self {
        Holder {
            raw: RawRef::clone_to(&value.raw, RefType::Holder).unwrap(),
        }
    }
}

impl<D: ?Sized> Drop for Holder<D> {
    fn drop(&mut self) {
        self.raw.drop_from(RefType::Holder);
    }
}

impl<D: ?Sized> Debug for Holder<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Holder").field(&self.raw).finish()
    }
}

impl<D: Default> Default for Holder<D> {
    fn default() -> Self {
        Holder::new(D::default())
    }
}

impl<D: ?Sized> PartialEq for Holder<D> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<D: ?Sized> Eq for Holder<D> {}

impl<D: ?Sized> Hash for Holder<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}
