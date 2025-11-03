use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;

use crate::Owner;
use crate::State;
use crate::Viewer;
use crate::ptr::Ptr;
use crate::ptr::Role;

pub struct Holder<D: ?Sized> {
    pub(crate) ptr: Ptr<D>,
}

impl<D: ?Sized> Holder<D> {
    pub fn new(d: D) -> Self
    where D: Sized {
        Holder { ptr: Ptr::new(d, Role::Holder) }
    }

    pub fn state(h: &Holder<D>) -> State {
        h.ptr.cell().state()
    }

    pub fn reinit(h: &Holder<D>, d: D) -> Result<(), State>
    where D: Sized {
        let state = h.ptr.cell().state();
        if state.is_dropped() {
            // SAFETY: data is dropped
            unsafe {
                h.ptr.cell().reinit_data(d);
            }
            Ok(())
        } else {
            Err(state)
        }
    }
}

impl<D: ?Sized> Clone for Holder<D> {
    fn clone(&self) -> Self {
        Holder { ptr: Ptr::clone_to(&self.ptr, Role::Holder).unwrap() }
    }
}

impl<D: ?Sized> From<&Viewer<D>> for Holder<D> {
    fn from(value: &Viewer<D>) -> Self {
        Holder { ptr: Ptr::clone_to(&value.ptr, Role::Holder).unwrap() }
    }
}

impl<D: ?Sized> From<Viewer<D>> for Holder<D> {
    fn from(value: Viewer<D>) -> Self {
        Holder { ptr: Ptr::clone_to(&value.ptr, Role::Holder).unwrap() }
    }
}

impl<D: ?Sized> From<&Owner<D>> for Holder<D> {
    fn from(value: &Owner<D>) -> Self {
        Holder { ptr: Ptr::clone_to(&value.ptr, Role::Holder).unwrap() }
    }
}

impl<D: ?Sized> From<Owner<D>> for Holder<D> {
    fn from(value: Owner<D>) -> Self {
        Holder { ptr: Ptr::clone_to(&value.ptr, Role::Holder).unwrap() }
    }
}

impl<D: ?Sized> Drop for Holder<D> {
    fn drop(&mut self) {
        self.ptr.drop_from(Role::Holder);
    }
}

impl<D: ?Sized> Debug for Holder<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Holder").field(&self.ptr).finish()
    }
}

impl<D: Default> Default for Holder<D> {
    fn default() -> Self {
        Holder::new(D::default())
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
