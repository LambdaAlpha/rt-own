use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

use crate::Holder;
use crate::Owner;
use crate::State;
use crate::ptr::Ptr;

pub struct Viewer<D: ?Sized> {
    ptr: Ptr<D>,
}

impl<D: ?Sized> Viewer<D> {
    pub fn new(data: D) -> Self
    where D: Sized {
        Self { ptr: Ptr::new_viewer(data) }
    }

    pub fn state(viewer: &Self) -> State {
        viewer.ptr.cell().state()
    }

    pub(crate) fn ptr(&self) -> &Ptr<D> {
        &self.ptr
    }
}

impl<D: ?Sized> Deref for Viewer<D> {
    type Target = D;
    fn deref(&self) -> &Self::Target {
        // SAFETY: when self is alive there is no owner and data hasn't been dropped
        unsafe { self.ptr.cell().deref() }
    }
}

impl<D: ?Sized> Clone for Viewer<D> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr.clone_to_viewer().unwrap() }
    }
}

impl<D: ?Sized> Drop for Viewer<D> {
    fn drop(&mut self) {
        self.ptr.drop_from_viewer();
    }
}

impl<D: ?Sized> TryFrom<&Holder<D>> for Viewer<D> {
    type Error = State;
    fn try_from(value: &Holder<D>) -> Result<Self, Self::Error> {
        Ok(Self { ptr: value.ptr().clone_to_viewer()? })
    }
}

impl<D: ?Sized> TryFrom<Holder<D>> for Viewer<D> {
    type Error = State;
    fn try_from(value: Holder<D>) -> Result<Self, Self::Error> {
        Ok(Self { ptr: value.ptr().clone_to_viewer()? })
    }
}

impl<D: ?Sized> From<Owner<D>> for Viewer<D> {
    fn from(value: Owner<D>) -> Self {
        let holder = Holder::from(value);
        Self { ptr: holder.ptr().clone_to_viewer().unwrap() }
    }
}

impl<D: ?Sized> Debug for Viewer<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Viewer").field(&self.ptr).finish()
    }
}

impl<D: Default> Default for Viewer<D> {
    fn default() -> Self {
        Self::new(D::default())
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
