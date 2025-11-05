use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;

use crate::Owner;
use crate::State;
use crate::Viewer;
use crate::ptr::Ptr;

pub struct Holder<D: ?Sized> {
    ptr: Ptr<D>,
}

impl<D: ?Sized> Holder<D> {
    pub fn new(data: D) -> Self
    where D: Sized {
        Self { ptr: Ptr::new_holder(data) }
    }

    pub fn state(holder: &Self) -> State {
        holder.ptr.cell().state()
    }

    pub(crate) fn ptr(&self) -> &Ptr<D> {
        &self.ptr
    }

    pub fn reinit(holder: &Self, data: D) -> Result<(), State>
    where D: Sized {
        let state = holder.ptr.cell().state();
        if state.is_dropped() {
            // SAFETY: data is dropped
            unsafe {
                holder.ptr.cell().reinit_data(data);
            }
            Ok(())
        } else {
            Err(state)
        }
    }
}

impl<D: ?Sized> Clone for Holder<D> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr.clone_to_holder() }
    }
}

impl<D: ?Sized> Drop for Holder<D> {
    fn drop(&mut self) {
        self.ptr.drop_from_holder();
    }
}

impl<D: ?Sized> From<&Viewer<D>> for Holder<D> {
    fn from(value: &Viewer<D>) -> Self {
        Self { ptr: value.ptr().clone_to_holder() }
    }
}

impl<D: ?Sized> From<Viewer<D>> for Holder<D> {
    fn from(value: Viewer<D>) -> Self {
        Self { ptr: value.ptr().clone_to_holder() }
    }
}

impl<D: ?Sized> From<&Owner<D>> for Holder<D> {
    fn from(value: &Owner<D>) -> Self {
        Self { ptr: value.ptr().clone_to_holder() }
    }
}

impl<D: ?Sized> From<Owner<D>> for Holder<D> {
    fn from(value: Owner<D>) -> Self {
        Self { ptr: value.ptr().clone_to_holder() }
    }
}

impl<D: ?Sized> Debug for Holder<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Holder").field(&self.ptr).finish()
    }
}

impl<D: Default> Default for Holder<D> {
    fn default() -> Self {
        Self::new(D::default())
    }
}

impl<D: ?Sized> PartialEq for Holder<D> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<D: ?Sized> Eq for Holder<D> {}

impl<D: ?Sized> Hash for Holder<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}
