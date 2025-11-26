use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;
use std::ops::DerefMut;
use std::ptr::NonNull;

use crate::Holder;
use crate::Owner;
use crate::State;
use crate::Viewer;
use crate::ptr::Ptr;
use crate::ref_::Ref;

pub struct OwnerRef<Source: ?Sized, Target: ?Sized> {
    ref_: Ref<Source, Target>,
}

impl<Source: ?Sized, Target: ?Sized> OwnerRef<Source, Target> {
    pub fn state(owner: &Self) -> State {
        owner.ref_.source().cell().state()
    }

    pub fn map<Target2, Map>(mut owner: Self, map: Map) -> OwnerRef<Source, Target2>
    where
        Target2: ?Sized + 'static,
        Map: for<'a> FnOnce(&'a mut Target) -> &'a mut Target2, {
        // SAFETY: when self is alive there is no owner and data hasn't been dropped
        let target = unsafe { owner.ref_.map_target_mut(map) };
        let holder = Holder::from(owner);
        let source = Holder::ptr(&holder).clone_to_owner().unwrap();
        OwnerRef { ref_: Ref::new(source, target) }
    }

    pub fn try_map<Target2, Err, Map>(
        mut owner: Self, map: Map,
    ) -> Result<OwnerRef<Source, Target2>, Err>
    where
        Target2: ?Sized + 'static,
        Map: for<'a> FnOnce(&'a mut Target) -> Result<&'a mut Target2, Err>, {
        // SAFETY: when self is alive there is no owner and data hasn't been dropped
        let target = unsafe { owner.ref_.try_map_target_mut(map) }?;
        let holder = Holder::from(owner);
        let source = Holder::ptr(&holder).clone_to_owner().unwrap();
        Ok(OwnerRef { ref_: Ref::new(source, target) })
    }

    pub(crate) fn source(owner: &Self) -> &Ptr<Source> {
        owner.ref_.source()
    }

    pub(crate) fn target(owner: &Self) -> NonNull<Target> {
        owner.ref_.target()
    }
}

impl<Source: ?Sized, Target: ?Sized> Drop for OwnerRef<Source, Target> {
    fn drop(&mut self) {
        self.ref_.source().drop_from_owner();
    }
}

impl<Source: ?Sized, Target: ?Sized> Deref for OwnerRef<Source, Target> {
    type Target = Target;
    fn deref(&self) -> &Self::Target {
        // SAFETY: we have exclusive ref and data hasn't been dropped
        unsafe { self.ref_.deref() }
    }
}

impl<Source: ?Sized, Target: ?Sized> DerefMut for OwnerRef<Source, Target> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: we have exclusive ref and data hasn't been dropped
        unsafe { self.ref_.deref_mut() }
    }
}

impl<Source: ?Sized> TryFrom<&Holder<Source>> for OwnerRef<Source, Source> {
    type Error = State;
    fn try_from(holder: &Holder<Source>) -> Result<Self, Self::Error> {
        let source = Holder::ptr(holder).clone_to_owner()?;
        Ok(Self { ref_: Ref::from_source(source) })
    }
}

impl<Source: ?Sized> TryFrom<Holder<Source>> for OwnerRef<Source, Source> {
    type Error = State;
    fn try_from(holder: Holder<Source>) -> Result<Self, Self::Error> {
        let source = Holder::ptr(&holder).clone_to_owner()?;
        Ok(Self { ref_: Ref::from_source(source) })
    }
}

impl<Source: ?Sized> TryFrom<Viewer<Source>> for OwnerRef<Source, Source> {
    type Error = State;
    fn try_from(value: Viewer<Source>) -> Result<Self, Self::Error> {
        let holder = Holder::from(value);
        let source = Holder::ptr(&holder).clone_to_owner()?;
        Ok(Self { ref_: Ref::from_source(source) })
    }
}

impl<Source: ?Sized> From<Owner<Source>> for OwnerRef<Source, Source> {
    fn from(value: Owner<Source>) -> Self {
        let holder = Holder::from(value);
        let source = Holder::ptr(&holder).clone_to_owner().unwrap();
        Self { ref_: Ref::from_source(source) }
    }
}

impl<Source: ?Sized, Target: ?Sized> Debug for OwnerRef<Source, Target> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Owner")?;
        self.ref_.fmt(f)
    }
}

impl<Source: ?Sized, Target: ?Sized> PartialEq for OwnerRef<Source, Target> {
    fn eq(&self, other: &Self) -> bool {
        self.ref_ == other.ref_
    }
}

impl<Source: ?Sized, Target: ?Sized> Eq for OwnerRef<Source, Target> {}

impl<Source: ?Sized, Target: ?Sized> Hash for OwnerRef<Source, Target> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ref_.hash(state);
    }
}
