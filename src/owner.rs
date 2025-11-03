use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::Holder;
use crate::Sharer;
use crate::State;
use crate::raw::RawRef;
use crate::raw::RefType;

pub struct Owner<D: ?Sized> {
    pub(crate) raw: RawRef<D>,
}

impl<D: ?Sized> Owner<D> {
    pub fn new(d: D) -> Self
    where D: Sized {
        Owner { raw: RawRef::new(d, RefType::Owner) }
    }

    pub fn state(o: &Owner<D>) -> State {
        o.raw.shared().state()
    }

    pub fn move_data(o: Owner<D>) -> D
    where D: Sized {
        // SAFETY:
        // we have exclusive ref
        // we consume the Owner when taking
        // we change the state to dropped
        // so we won't access the data anymore
        unsafe { o.raw.shared().move_data() }
    }

    pub fn drop_data(o: Owner<D>) {
        // SAFETY:
        // we have exclusive ref
        // we consume the Owner when deleting
        // we change the state to dropped
        // so we won't access the data anymore
        unsafe { o.raw.shared().drop_data() }
    }
}

impl<D: ?Sized> Deref for Owner<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        // SAFETY: we have exclusive ref and data hasn't been dropped
        unsafe { self.raw.shared().deref() }
    }
}

impl<D: ?Sized> DerefMut for Owner<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: we have exclusive ref and data hasn't been dropped
        unsafe { self.raw.shared().deref_mut() }
    }
}

impl<D: ?Sized> TryFrom<&Holder<D>> for Owner<D> {
    type Error = State;
    fn try_from(value: &Holder<D>) -> Result<Self, Self::Error> {
        Ok(Owner { raw: RawRef::clone_to(&value.raw, RefType::Owner)? })
    }
}

impl<D: ?Sized> TryFrom<Holder<D>> for Owner<D> {
    type Error = State;
    fn try_from(value: Holder<D>) -> Result<Self, Self::Error> {
        Ok(Owner { raw: RawRef::clone_to(&value.raw, RefType::Owner)? })
    }
}

impl<D: ?Sized> TryFrom<Sharer<D>> for Owner<D> {
    type Error = State;
    fn try_from(value: Sharer<D>) -> Result<Self, Self::Error> {
        let h = Holder::from(value);
        Ok(Owner { raw: RawRef::clone_to(&h.raw, RefType::Owner)? })
    }
}

impl<D: ?Sized> Drop for Owner<D> {
    fn drop(&mut self) {
        self.raw.drop_from(RefType::Owner);
    }
}

impl<D: ?Sized> Debug for Owner<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Owner").field(&self.raw).finish()
    }
}

impl<D: Default> Default for Owner<D> {
    fn default() -> Self {
        Owner::new(D::default())
    }
}

impl<D: ?Sized> PartialEq for Owner<D> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<D: ?Sized> Eq for Owner<D> {}

impl<D: ?Sized> Hash for Owner<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}
