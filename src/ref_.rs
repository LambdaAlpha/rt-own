use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ptr;
use std::ptr::NonNull;

use crate::ptr::Ptr;

pub(crate) struct Ref<Source: ?Sized, Target: ?Sized> {
    source: Ptr<Source>,
    target: NonNull<Target>,
}

impl<Source: ?Sized, Target: ?Sized> Ref<Source, Target> {
    pub(crate) fn new(source: Ptr<Source>, target: NonNull<Target>) -> Self {
        Self { source, target }
    }

    pub(crate) fn source(&self) -> &Ptr<Source> {
        &self.source
    }

    pub(crate) fn target(&self) -> NonNull<Target> {
        self.target
    }

    // SAFETY: make sure data not dropped and there is no mut ref
    pub(crate) unsafe fn map_target<Target2, Map>(&self, map: Map) -> NonNull<Target2>
    where
        Target2: ?Sized + 'static,
        Map: for<'a> FnOnce(&'a Target) -> &'a Target2, {
        // SAFETY: make sure data not dropped and there is no mut ref
        let target = unsafe { self.target.as_ref() };
        NonNull::from_ref(map(target))
    }

    // SAFETY: make sure data not dropped and there is no ref
    pub(crate) unsafe fn map_target_mut<Target2, Map>(&mut self, map: Map) -> NonNull<Target2>
    where
        Target2: ?Sized + 'static,
        Map: for<'a> FnOnce(&'a mut Target) -> &'a mut Target2, {
        // SAFETY: make sure data not dropped and there is no ref
        let target = unsafe { self.target.as_mut() };
        NonNull::from_ref(map(target))
    }

    // SAFETY: make sure data not dropped and there is no mut ref
    pub(crate) unsafe fn deref<'a>(&self) -> &'a Target {
        // SAFETY: make sure data not dropped and there is no mut ref
        unsafe { self.target.as_ref() }
    }

    // SAFETY: make sure data not dropped and there is no ref
    pub(crate) unsafe fn deref_mut<'a>(&mut self) -> &'a mut Target {
        // SAFETY: make sure data not dropped and there is no ref
        unsafe { self.target.as_mut() }
    }
}

impl<Source: ?Sized> Ref<Source, Source> {
    pub(crate) fn from_source(source: Ptr<Source>) -> Self {
        let target = NonNull::new(source.cell().data_ptr()).unwrap();
        Self { source, target }
    }
}

impl<Source: ?Sized, Target: ?Sized> Debug for Ref<Source, Target> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Ref").field(&self.source).field(&self.target).finish()
    }
}

impl<Source: ?Sized, Target: ?Sized> PartialEq for Ref<Source, Target> {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && ptr::addr_eq(self.target.as_ptr(), other.target.as_ptr())
    }
}

impl<Source: ?Sized, Target: ?Sized> Eq for Ref<Source, Target> {}

impl<Source: ?Sized, Target: ?Sized> Hash for Ref<Source, Target> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.target.hash(state);
    }
}
