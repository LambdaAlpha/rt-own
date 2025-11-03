use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

use crate::Holder;
use crate::Owner;
use crate::State;
use crate::ptr::Ptr;
use crate::ptr::Role;

pub struct Viewer<D: ?Sized> {
    pub(crate) ptr: Ptr<D>,
}

impl<D: ?Sized> Viewer<D> {
    pub fn new(d: D) -> Self
    where D: Sized {
        Viewer { ptr: Ptr::new(d, Role::Viewer) }
    }

    pub fn state(s: &Viewer<D>) -> State {
        s.ptr.cell().state()
    }
}

impl<D: ?Sized> Clone for Viewer<D> {
    fn clone(&self) -> Self {
        Viewer { ptr: Ptr::clone_to(&self.ptr, Role::Viewer).unwrap() }
    }
}

impl<D: ?Sized> TryFrom<&Holder<D>> for Viewer<D> {
    type Error = State;
    fn try_from(value: &Holder<D>) -> Result<Self, Self::Error> {
        Ok(Viewer { ptr: Ptr::clone_to(&value.ptr, Role::Viewer)? })
    }
}

impl<D: ?Sized> TryFrom<Holder<D>> for Viewer<D> {
    type Error = State;
    fn try_from(value: Holder<D>) -> Result<Self, Self::Error> {
        Ok(Viewer { ptr: Ptr::clone_to(&value.ptr, Role::Viewer)? })
    }
}

impl<D: ?Sized> From<Owner<D>> for Viewer<D> {
    fn from(value: Owner<D>) -> Self {
        let h = Holder::from(value);
        Viewer { ptr: Ptr::clone_to(&h.ptr, Role::Viewer).unwrap() }
    }
}

impl<D: ?Sized> Deref for Viewer<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        // SAFETY: when self is alive there is no owner and data hasn't been dropped
        unsafe { self.ptr.cell().deref() }
    }
}

impl<D: ?Sized> TryFrom<&Ptr<D>> for Viewer<D> {
    type Error = State;
    fn try_from(value: &Ptr<D>) -> Result<Self, Self::Error> {
        Ok(Viewer { ptr: Ptr::clone_to(value, Role::Viewer)? })
    }
}

impl<D: ?Sized> Drop for Viewer<D> {
    fn drop(&mut self) {
        self.ptr.drop_from(Role::Viewer);
    }
}

impl<D: ?Sized> Debug for Viewer<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Viewer").field(&self.ptr).finish()
    }
}

impl<D: Default> Default for Viewer<D> {
    fn default() -> Self {
        Viewer::new(D::default())
    }
}

impl<D: ?Sized> PartialEq for Viewer<D> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<D: ?Sized> Eq for Viewer<D> {}

impl<D: ?Sized> Hash for Viewer<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}
