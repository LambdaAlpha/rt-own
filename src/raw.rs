use std::alloc;
use std::cell::Cell;
use std::cell::UnsafeCell;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::ptr;
use std::ptr::NonNull;

pub(crate) struct RawRef<D: ?Sized> {
    ptr: NonNull<SharedCell<D>>,
    phantom: PhantomData<SharedCell<D>>,
}

impl<D: ?Sized> RawRef<D> {
    pub(crate) fn new(d: D, ref_type: RefType) -> Self
    where D: Sized {
        RawRef {
            ptr: Box::leak(Box::new(SharedCell::new(d, ref_type))).into(),
            phantom: PhantomData,
        }
    }

    pub(crate) fn clone_to(&self, ref_type: RefType) -> Result<RawRef<D>, State> {
        self.shared().clone_to(ref_type)?;
        Ok(RawRef { ptr: self.ptr, phantom: PhantomData })
    }

    pub(crate) fn drop_from(&self, ref_type: RefType) {
        self.shared().drop_from(ref_type);

        if self.shared().should_dealloc() {
            let layout = alloc::Layout::for_value(self.shared());
            // SAFETY:
            // state promises that we can and should dealloc
            // we are the last RawCell accessible to the ptr of shared cell, and we are dropped
            // we carefully don't make any ref to shared cell when calling dealloc
            unsafe {
                alloc::dealloc(self.ptr.as_ptr().cast(), layout);
            }
        }
    }

    pub(crate) fn shared(&self) -> &SharedCell<D> {
        // SAFETY: when self is alive, ptr is always valid, and we never call ptr.as_mut()
        unsafe { self.ptr.as_ref() }
    }
}

impl<D: ?Sized> Debug for RawRef<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.ptr.fmt(f)
    }
}

impl<D: ?Sized> PartialEq for RawRef<D> {
    fn eq(&self, other: &Self) -> bool {
        ptr::addr_eq(self.ptr.as_ptr(), other.ptr.as_ptr())
    }
}

impl<D: ?Sized> Eq for RawRef<D> {}

impl<D: ?Sized> Hash for RawRef<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

pub(crate) struct SharedCell<D: ?Sized> {
    state: Cell<State>,
    data: UnsafeCell<D>,
}

impl<D: ?Sized> SharedCell<D> {
    fn new(d: D, ref_type: RefType) -> Self
    where D: Sized {
        SharedCell { state: Cell::new(State::new(ref_type)), data: UnsafeCell::new(d) }
    }

    pub(crate) fn state(&self) -> State {
        self.state.get()
    }

    fn clone_to(&self, ref_type: RefType) -> Result<(), State> {
        self.state.set(self.state.get().clone_to(ref_type)?);
        Ok(())
    }

    fn drop_from(&self, ref_type: RefType) {
        self.state.set(self.state.get().drop_from(ref_type));
        if self.state.get().should_drop() {
            // SAFETY: state promises that we can and should drop
            unsafe { self.drop_data() }
        }
    }

    // SAFETY: call only once and there is no ref
    pub(crate) unsafe fn move_data(&self) -> D
    where D: Sized {
        self.state.set(self.state.get().drop());
        // SAFETY: call only once and there is no ref
        unsafe { ptr::read(self.data.get()) }
    }

    // SAFETY: call only once and there is no ref
    pub(crate) unsafe fn drop_data(&self) {
        self.state.set(self.state.get().drop());
        // SAFETY: call only once and there is no ref
        unsafe {
            ptr::drop_in_place(self.data.get());
        }
    }

    // SAFETY: data is dropped
    pub(crate) unsafe fn reinit_data(&self, d: D)
    where D: Sized {
        self.state.set(self.state.get().reinit());
        // SAFETY: data is dropped
        unsafe {
            ptr::write(self.data.get(), d);
        }
    }

    fn should_dealloc(&self) -> bool {
        self.state.get().should_dealloc()
    }

    // SAFETY: make sure data not dropped and there is no owner
    pub(crate) unsafe fn deref<'a>(&self) -> &'a D {
        // SAFETY: make sure data not dropped and there is no owner
        unsafe { self.data.get().as_ref().unwrap() }
    }

    // SAFETY: make sure data not dropped and there is no ref
    pub(crate) unsafe fn deref_mut<'a>(&self) -> &'a mut D {
        // SAFETY: make sure data not dropped and there is no ref
        unsafe { self.data.get().as_mut().unwrap() }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum RefType {
    Holder,
    Sharer,
    Owner,
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub struct State {
    // the sign bit indicates whether data has been dropped (negative)
    // other bits indicates holder cnt
    holder_cnt: isize,
    // the sign bit indicates whether data has been owned (negative)
    // other bits indicates sharer cnt
    sharer_cnt: isize,
}

impl State {
    pub fn is_dropped(&self) -> bool {
        self.holder_cnt < 0
    }
    pub fn holder_count(&self) -> usize {
        (self.holder_cnt & isize::MAX) as usize
    }
    pub fn sharer_count(&self) -> usize {
        (self.sharer_cnt & isize::MAX) as usize
    }
    pub fn is_owned(&self) -> bool {
        self.sharer_cnt < 0
    }

    fn new(ref_type: RefType) -> Self {
        let (keep_cnt, read_cnt) = match ref_type {
            RefType::Holder => (1, 0),
            RefType::Sharer => (0, 1),
            RefType::Owner => (0, isize::MIN),
        };
        State { holder_cnt: keep_cnt, sharer_cnt: read_cnt }
    }

    fn clone_to(mut self, ref_type: RefType) -> Result<State, State> {
        match ref_type {
            RefType::Holder => {
                self.holder_cnt += 1;
                Ok(self)
            }
            RefType::Sharer => {
                if self.is_dropped() || self.is_owned() {
                    Err(self)
                } else {
                    self.sharer_cnt += 1;
                    Ok(self)
                }
            }
            RefType::Owner => {
                if self.is_dropped() || self.sharer_cnt != 0 {
                    Err(self)
                } else {
                    self.sharer_cnt = isize::MIN;
                    Ok(self)
                }
            }
        }
    }

    fn drop_from(mut self, ref_type: RefType) -> State {
        match ref_type {
            RefType::Holder => self.holder_cnt -= 1,
            RefType::Sharer => self.sharer_cnt -= 1,
            RefType::Owner => self.sharer_cnt = 0,
        }
        self
    }

    fn drop(mut self) -> State {
        self.holder_cnt |= isize::MIN;
        self
    }

    fn reinit(mut self) -> State {
        self.holder_cnt &= isize::MAX;
        self
    }

    // if already dropped, return false
    fn should_drop(&self) -> bool {
        self.holder_cnt == 0 && self.sharer_cnt == 0
    }

    fn should_dealloc(&self) -> bool {
        self.holder_cnt == isize::MIN && self.sharer_cnt == 0
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("dropped", &self.is_dropped())
            .field("holder", &self.holder_count())
            .field("sharer", &self.sharer_count())
            .field("owned", &self.is_owned())
            .finish()
    }
}
