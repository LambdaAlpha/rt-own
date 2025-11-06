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

pub(crate) struct Ptr<D: ?Sized> {
    ptr: NonNull<StateCell<D>>,
    phantom: PhantomData<StateCell<D>>,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct State {
    // the sign bit indicates whether data has been dropped (negative)
    // other bits indicates holder cnt
    holder_cnt: isize,
    // the sign bit indicates whether data has been owned (negative)
    // other bits indicates viewer cnt
    viewer_cnt: isize,
}

impl<D: ?Sized> Ptr<D> {
    pub(crate) fn new_holder(data: D) -> Self
    where D: Sized {
        Self::new(data, State::new_holder())
    }

    pub(crate) fn new_viewer(data: D) -> Self
    where D: Sized {
        Self::new(data, State::new_viewer())
    }

    pub(crate) fn new_owner(data: D) -> Self
    where D: Sized {
        Self::new(data, State::new_owner())
    }

    fn new(data: D, state: State) -> Self
    where D: Sized {
        let ptr = Box::leak(Box::new(StateCell::new(data, state)));
        Ptr { ptr: NonNull::from_mut(ptr), phantom: PhantomData }
    }

    pub(crate) fn clone_to_holder(&self) -> Self {
        self.cell().clone_to_holder();
        Ptr { ptr: self.ptr, phantom: PhantomData }
    }

    pub(crate) fn clone_to_viewer(&self) -> Result<Self, State> {
        self.cell().clone_to_viewer()?;
        Ok(Ptr { ptr: self.ptr, phantom: PhantomData })
    }

    pub(crate) fn clone_to_owner(&self) -> Result<Self, State> {
        self.cell().clone_to_owner()?;
        Ok(Ptr { ptr: self.ptr, phantom: PhantomData })
    }

    pub(crate) fn drop_from_holder(&self) {
        self.cell().drop_from_holder();
        self.check_dealloc();
    }

    pub(crate) fn drop_from_viewer(&self) {
        self.cell().drop_from_viewer();
        self.check_dealloc();
    }

    pub(crate) fn drop_from_owner(&self) {
        self.cell().drop_from_owner();
        self.check_dealloc();
    }

    fn check_dealloc(&self) {
        if self.cell().should_dealloc() {
            let layout = alloc::Layout::for_value(self.cell());
            // SAFETY:
            // state promises that we can and should dealloc
            // we are the last Ptr accessible to the ptr of PtrCell, and we are dropped
            // we carefully don't make any ref to PtrCell when calling dealloc
            unsafe {
                alloc::dealloc(self.ptr.as_ptr().cast(), layout);
            }
        }
    }

    pub(crate) fn cell(&self) -> &StateCell<D> {
        // SAFETY: when self is alive, ptr is always valid, and we never call ptr.as_mut()
        unsafe { self.ptr.as_ref() }
    }
}

impl<D: ?Sized> Debug for Ptr<D> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.ptr.fmt(f)
    }
}

impl<D: ?Sized> PartialEq for Ptr<D> {
    fn eq(&self, other: &Self) -> bool {
        ptr::addr_eq(self.ptr.as_ptr(), other.ptr.as_ptr())
    }
}

impl<D: ?Sized> Eq for Ptr<D> {}

impl<D: ?Sized> Hash for Ptr<D> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

pub(crate) struct StateCell<D: ?Sized> {
    state: Cell<State>,
    data: UnsafeCell<D>,
}

impl<D: ?Sized> StateCell<D> {
    fn new(data: D, state: State) -> Self
    where D: Sized {
        StateCell { state: Cell::new(state), data: UnsafeCell::new(data) }
    }

    pub(crate) fn state(&self) -> State {
        self.state.get()
    }

    fn clone_to_holder(&self) {
        self.state.set(self.state.get().clone_to_holder());
    }

    fn clone_to_viewer(&self) -> Result<(), State> {
        self.state.set(self.state.get().clone_to_viewer()?);
        Ok(())
    }

    fn clone_to_owner(&self) -> Result<(), State> {
        self.state.set(self.state.get().clone_to_owner()?);
        Ok(())
    }

    fn drop_from_holder(&self) {
        self.state.set(self.state.get().drop_from_holder());
        self.check_drop_data();
    }

    fn drop_from_viewer(&self) {
        self.state.set(self.state.get().drop_from_viewer());
        self.check_drop_data();
    }

    fn drop_from_owner(&self) {
        self.state.set(self.state.get().drop_from_owner());
        self.check_drop_data();
    }

    fn check_drop_data(&self) {
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

    // SAFETY: make sure data not dropped and there is no mut ref
    pub(crate) unsafe fn deref<'a>(&self) -> &'a D {
        // SAFETY: make sure data not dropped and there is no mut ref
        unsafe { self.data.get().as_ref().unwrap() }
    }

    // SAFETY: make sure data not dropped and there is no ref
    pub(crate) unsafe fn deref_mut<'a>(&self) -> &'a mut D {
        // SAFETY: make sure data not dropped and there is no ref
        unsafe { self.data.get().as_mut().unwrap() }
    }

    pub(crate) fn data_ptr(&self) -> *mut D {
        self.data.get()
    }
}

impl State {
    pub fn is_dropped(&self) -> bool {
        self.holder_cnt < 0
    }

    pub fn holder_count(&self) -> usize {
        (self.holder_cnt & isize::MAX) as usize
    }

    pub fn viewer_count(&self) -> usize {
        (self.viewer_cnt & isize::MAX) as usize
    }

    pub fn is_owned(&self) -> bool {
        self.viewer_cnt < 0
    }

    fn new_holder() -> Self {
        Self { holder_cnt: 1, viewer_cnt: 0 }
    }

    fn new_viewer() -> Self {
        Self { holder_cnt: 0, viewer_cnt: 1 }
    }

    fn new_owner() -> Self {
        Self { holder_cnt: 0, viewer_cnt: isize::MIN }
    }

    fn clone_to_holder(mut self) -> Self {
        self.holder_cnt += 1;
        self
    }

    fn clone_to_viewer(mut self) -> Result<Self, Self> {
        if self.is_dropped() || self.is_owned() {
            Err(self)
        } else {
            self.viewer_cnt += 1;
            Ok(self)
        }
    }

    fn clone_to_owner(mut self) -> Result<Self, Self> {
        if self.is_dropped() || self.viewer_cnt != 0 {
            Err(self)
        } else {
            self.viewer_cnt = isize::MIN;
            Ok(self)
        }
    }

    fn drop_from_holder(mut self) -> Self {
        self.holder_cnt -= 1;
        self
    }

    fn drop_from_viewer(mut self) -> Self {
        self.viewer_cnt -= 1;
        self
    }

    fn drop_from_owner(mut self) -> Self {
        self.viewer_cnt = 0;
        self
    }

    fn drop(mut self) -> Self {
        self.holder_cnt |= isize::MIN;
        self
    }

    fn reinit(mut self) -> Self {
        self.holder_cnt &= isize::MAX;
        self
    }

    // if already dropped, return false
    fn should_drop(&self) -> bool {
        self.holder_cnt == 0 && self.viewer_cnt == 0
    }

    fn should_dealloc(&self) -> bool {
        self.holder_cnt == isize::MIN && self.viewer_cnt == 0
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("dropped", &self.is_dropped())
            .field("holder", &self.holder_count())
            .field("viewer", &self.viewer_count())
            .field("owned", &self.is_owned())
            .finish()
    }
}
