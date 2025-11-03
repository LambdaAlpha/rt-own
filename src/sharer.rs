use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

use crate::Holder;
use crate::Owner;
use crate::State;
use crate::raw::RawRef;
use crate::raw::RefType;

pub struct Sharer<D: ?Sized> {
    pub(crate) raw: RawRef<D>,
}

impl<D: ?Sized> Sharer<D> {
    pub fn new(d: D) -> Self
    where D: Sized {
        Sharer { raw: RawRef::new(d, RefType::Sharer) }
    }

    pub fn state(s: &Sharer<D>) -> State {
        s.raw.shared().state()
    }
}

impl<D: ?Sized> Clone for Sharer<D> {
    fn clone(&self) -> Self {
        Sharer { raw: RawRef::clone_to(&self.raw, RefType::Sharer).unwrap() }
    }
}

impl<D: ?Sized> TryFrom<&Holder<D>> for Sharer<D> {
    type Error = State;
    fn try_from(value: &Holder<D>) -> Result<Self, Self::Error> {
        Ok(Sharer { raw: RawRef::clone_to(&value.raw, RefType::Sharer)? })
    }
}

impl<D: ?Sized> TryFrom<Holder<D>> for Sharer<D> {
    type Error = State;
    fn try_from(value: Holder<D>) -> Result<Self, Self::Error> {
        Ok(Sharer { raw: RawRef::clone_to(&value.raw, RefType::Sharer)? })
    }
}

impl<D: ?Sized> From<Owner<D>> for Sharer<D> {
    fn from(value: Owner<D>) -> Self {
        let h = Holder::from(value);
        Sharer { raw: RawRef::clone_to(&h.raw, RefType::Sharer).unwrap() }
    }
}

impl<D: ?Sized> Deref for Sharer<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        // SAFETY: when self is alive there is no owner and data hasn't been dropped
        unsafe { self.raw.shared().deref() }
    }
}

impl<D: ?Sized> TryFrom<&RawRef<D>> for Sharer<D> {
    type Error = State;
    fn try_from(value: &RawRef<D>) -> Result<Self, Self::Error> {
        Ok(Sharer { raw: RawRef::clone_to(value, RefType::Sharer)? })
    }
}

impl<D: ?Sized> Drop for Sharer<D> {
    fn drop(&mut self) {
        self.raw.drop_from(RefType::Sharer);
    }
}

impl<D: ?Sized> Debug for Sharer<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Sharer").field(&self.raw).finish()
    }
}

impl<D: Default> Default for Sharer<D> {
    fn default() -> Self {
        Sharer::new(D::default())
    }
}

impl<D: ?Sized> PartialEq for Sharer<D> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<D: ?Sized> Eq for Sharer<D> {}

impl<D: ?Sized> Hash for Sharer<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}
