use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::ops::Deref;

use crate::Holder;
use crate::Owner;
use crate::OwnerRef;
use crate::State;
use crate::Viewer;
use crate::ptr::Ptr;
use crate::ref_::Ref;

pub struct ViewerRef<Source: ?Sized, Target: ?Sized> {
    ref_: Ref<Source, Target>,
}

impl<Source: ?Sized, Target: ?Sized> ViewerRef<Source, Target> {
    pub fn state(viewer: &Self) -> State {
        viewer.ref_.source().cell().state()
    }

    pub fn map<Target2, Map>(viewer: Self, map: Map) -> ViewerRef<Source, Target2>
    where
        Target2: ?Sized + 'static,
        Map: for<'a> FnOnce(&'a Target) -> &'a Target2, {
        // SAFETY: when self is alive there is no owner and data hasn't been dropped
        let target = unsafe { viewer.ref_.map_target(map) };
        let source = viewer.ref_.source().clone_to_viewer().unwrap();
        ViewerRef { ref_: Ref::new(source, target) }
    }

    pub(crate) fn source(viewer: &Self) -> &Ptr<Source> {
        viewer.ref_.source()
    }
}

impl<Source: ?Sized, Target: ?Sized> Clone for ViewerRef<Source, Target> {
    fn clone(&self) -> Self {
        let source = self.ref_.source().clone_to_viewer().unwrap();
        let target = self.ref_.target();
        Self { ref_: Ref::new(source, target) }
    }
}

impl<Source: ?Sized, Target: ?Sized> Drop for ViewerRef<Source, Target> {
    fn drop(&mut self) {
        self.ref_.source().drop_from_viewer();
    }
}

impl<Source: ?Sized, Target: ?Sized> Deref for ViewerRef<Source, Target> {
    type Target = Target;
    fn deref(&self) -> &Self::Target {
        // SAFETY: when self is alive there is no owner and data hasn't been dropped
        unsafe { self.ref_.deref() }
    }
}

impl<Source: ?Sized> TryFrom<&Holder<Source>> for ViewerRef<Source, Source> {
    type Error = State;
    fn try_from(holder: &Holder<Source>) -> Result<Self, Self::Error> {
        let source = Holder::ptr(holder).clone_to_viewer()?;
        Ok(Self { ref_: Ref::from_source(source) })
    }
}

impl<Source: ?Sized> TryFrom<Holder<Source>> for ViewerRef<Source, Source> {
    type Error = State;
    fn try_from(holder: Holder<Source>) -> Result<Self, Self::Error> {
        let source = Holder::ptr(&holder).clone_to_viewer()?;
        Ok(Self { ref_: Ref::from_source(source) })
    }
}

impl<Source: ?Sized> From<&Viewer<Source>> for ViewerRef<Source, Source> {
    fn from(value: &Viewer<Source>) -> Self {
        let source = Viewer::ptr(value).clone_to_viewer().unwrap();
        Self { ref_: Ref::from_source(source) }
    }
}

impl<Source: ?Sized> From<Viewer<Source>> for ViewerRef<Source, Source> {
    fn from(value: Viewer<Source>) -> Self {
        let source = Viewer::ptr(&value).clone_to_viewer().unwrap();
        Self { ref_: Ref::from_source(source) }
    }
}

impl<Source: ?Sized> From<Owner<Source>> for ViewerRef<Source, Source> {
    fn from(value: Owner<Source>) -> Self {
        let holder = Holder::from(value);
        let source = Holder::ptr(&holder).clone_to_viewer().unwrap();
        Self { ref_: Ref::from_source(source) }
    }
}

impl<Source: ?Sized, Target: ?Sized> From<OwnerRef<Source, Target>> for ViewerRef<Source, Target> {
    fn from(value: OwnerRef<Source, Target>) -> Self {
        let target = OwnerRef::target(&value);
        let holder = Holder::from(value);
        let source = Holder::ptr(&holder).clone_to_viewer().unwrap();
        ViewerRef { ref_: Ref::new(source, target) }
    }
}

impl<Source: ?Sized, Target: ?Sized> Debug for ViewerRef<Source, Target> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Viewer")?;
        self.ref_.fmt(f)
    }
}

impl<Source: ?Sized, Target: ?Sized> PartialEq for ViewerRef<Source, Target> {
    fn eq(&self, other: &Self) -> bool {
        self.ref_ == other.ref_
    }
}

impl<Source: ?Sized, Target: ?Sized> Eq for ViewerRef<Source, Target> {}

impl<Source: ?Sized, Target: ?Sized> Hash for ViewerRef<Source, Target> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ref_.hash(state);
    }
}
