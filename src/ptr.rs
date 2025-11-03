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

#[derive(Debug, Copy, Clone)]
pub(crate) enum Role {
    Holder,
    Viewer,
    Owner,
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
    pub(crate) fn new(d: D, role: Role) -> Self
    where D: Sized {
        let ptr = Box::leak(Box::new(StateCell::new(d, role)));
        Ptr { ptr: ptr.into(), phantom: PhantomData }
    }

    pub(crate) fn clone_to(&self, role: Role) -> Result<Ptr<D>, State> {
        self.cell().clone_to(role)?;
        Ok(Ptr { ptr: self.ptr, phantom: PhantomData })
    }

    pub(crate) fn drop_from(&self, role: Role) {
        self.cell().drop_from(role);

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
    fn new(d: D, role: Role) -> Self
    where D: Sized {
        StateCell { state: Cell::new(State::new(role)), data: UnsafeCell::new(d) }
    }

    pub(crate) fn state(&self) -> State {
        self.state.get()
    }

    fn clone_to(&self, role: Role) -> Result<(), State> {
        self.state.set(self.state.get().clone_to(role)?);
        Ok(())
    }

    fn drop_from(&self, role: Role) {
        self.state.set(self.state.get().drop_from(role));
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

    fn new(role: Role) -> Self {
        let (holder_cnt, viewer_cnt) = match role {
            Role::Holder => (1, 0),
            Role::Viewer => (0, 1),
            Role::Owner => (0, isize::MIN),
        };
        State { holder_cnt, viewer_cnt }
    }

    fn clone_to(mut self, role: Role) -> Result<State, State> {
        match role {
            Role::Holder => {
                self.holder_cnt += 1;
                Ok(self)
            }
            Role::Viewer => {
                if self.is_dropped() || self.is_owned() {
                    Err(self)
                } else {
                    self.viewer_cnt += 1;
                    Ok(self)
                }
            }
            Role::Owner => {
                if self.is_dropped() || self.viewer_cnt != 0 {
                    Err(self)
                } else {
                    self.viewer_cnt = isize::MIN;
                    Ok(self)
                }
            }
        }
    }

    fn drop_from(mut self, role: Role) -> State {
        match role {
            Role::Holder => self.holder_cnt -= 1,
            Role::Viewer => self.viewer_cnt -= 1,
            Role::Owner => self.viewer_cnt = 0,
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
