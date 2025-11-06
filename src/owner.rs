use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::Holder;
use crate::OwnerRef;
use crate::State;
use crate::Viewer;
use crate::ViewerRef;
use crate::ptr::Ptr;

pub struct Owner<D: ?Sized> {
    ptr: Ptr<D>,
}

impl<D: ?Sized> Owner<D> {
    pub fn new(data: D) -> Self
    where D: Sized {
        Self { ptr: Ptr::new_owner(data) }
    }

    pub fn state(owner: &Self) -> State {
        owner.ptr.cell().state()
    }

    pub fn move_data(owner: Self) -> D
    where D: Sized {
        // SAFETY:
        // we have exclusive ref
        // we consume the Owner when taking
        // we change the state to dropped
        // so we won't access the data anymore
        unsafe { owner.ptr.cell().move_data() }
    }

    pub fn drop_data(owner: Self) {
        // SAFETY:
        // we have exclusive ref
        // we consume the Owner when deleting
        // we change the state to dropped
        // so we won't access the data anymore
        unsafe { owner.ptr.cell().drop_data() }
    }

    pub(crate) fn ptr(owner: &Self) -> &Ptr<D> {
        &owner.ptr
    }
}

impl<D: ?Sized> Deref for Owner<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        // SAFETY: we have exclusive ref and data hasn't been dropped
        unsafe { self.ptr.cell().deref() }
    }
}

impl<D: ?Sized> DerefMut for Owner<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: we have exclusive ref and data hasn't been dropped
        unsafe { self.ptr.cell().deref_mut() }
    }
}

impl<D: ?Sized> Drop for Owner<D> {
    fn drop(&mut self) {
        self.ptr.drop_from_owner();
    }
}

impl<D: ?Sized> TryFrom<&Holder<D>> for Owner<D> {
    type Error = State;
    fn try_from(value: &Holder<D>) -> Result<Self, Self::Error> {
        Ok(Self { ptr: Holder::ptr(value).clone_to_owner()? })
    }
}

impl<D: ?Sized> TryFrom<Holder<D>> for Owner<D> {
    type Error = State;
    fn try_from(value: Holder<D>) -> Result<Self, Self::Error> {
        Ok(Self { ptr: Holder::ptr(&value).clone_to_owner()? })
    }
}

impl<D: ?Sized> TryFrom<Viewer<D>> for Owner<D> {
    type Error = State;
    fn try_from(value: Viewer<D>) -> Result<Self, Self::Error> {
        let holder = Holder::from(value);
        Ok(Self { ptr: Holder::ptr(&holder).clone_to_owner()? })
    }
}

impl<Source: ?Sized, Target: ?Sized> TryFrom<ViewerRef<Source, Target>> for Owner<Source> {
    type Error = State;
    fn try_from(value: ViewerRef<Source, Target>) -> Result<Self, Self::Error> {
        let holder = Holder::from(value);
        Ok(Self { ptr: Holder::ptr(&holder).clone_to_owner()? })
    }
}

impl<Source: ?Sized, Target: ?Sized> From<OwnerRef<Source, Target>> for Owner<Source> {
    fn from(value: OwnerRef<Source, Target>) -> Self {
        let holder = Holder::from(value);
        Self { ptr: Holder::ptr(&holder).clone_to_owner().unwrap() }
    }
}

impl<D: ?Sized> Debug for Owner<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Owner").field(&self.ptr).finish()
    }
}

impl<D: Default> Default for Owner<D> {
    fn default() -> Self {
        Self::new(D::default())
    }
}

impl<D: ?Sized> PartialEq for Owner<D> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<D: ?Sized> Eq for Owner<D> {}

impl<D: ?Sized> Hash for Owner<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}
