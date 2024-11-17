use crate::gl::GL;
use std::{
    cell::UnsafeCell,
    panic::{RefUnwindSafe, UnwindSafe},
    sync::{Arc, RwLock},
};

pub struct LazyInitialized {
    pub memory: wasmer::Memory,
}
#[derive(Clone)]
pub struct EnvWrapper {
    pub gl: Arc<RwLock<GL>>,
    pub lazy: std::rc::Rc<OnceCell<LazyInitialized>>,
}

unsafe impl Send for EnvWrapper {}

unsafe impl Sync for EnvWrapper {}

pub struct OnceCell<T> {
    // Invariant: written to at most once.
    inner: UnsafeCell<Option<T>>,
}

impl<T: RefUnwindSafe + UnwindSafe> RefUnwindSafe for OnceCell<T> {}
impl<T: UnwindSafe> UnwindSafe for OnceCell<T> {}

impl<T> Default for OnceCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for OnceCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.get() {
            Some(v) => f.debug_tuple("OnceCell").field(v).finish(),
            None => f.write_str("OnceCell(Uninit)"),
        }
    }
}

impl<T: Clone> Clone for OnceCell<T> {
    fn clone(&self) -> OnceCell<T> {
        match self.get() {
            Some(value) => OnceCell::with_value(value.clone()),
            None => OnceCell::new(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self.get_mut(), source.get()) {
            (Some(this), Some(source)) => this.clone_from(source),
            _ => *self = source.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for OnceCell<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for OnceCell<T> {}

impl<T> From<T> for OnceCell<T> {
    fn from(value: T) -> Self {
        OnceCell::with_value(value)
    }
}

impl<T> OnceCell<T> {
    pub const fn new() -> OnceCell<T> {
        OnceCell {
            inner: UnsafeCell::new(None),
        }
    }

    pub const fn with_value(value: T) -> OnceCell<T> {
        OnceCell {
            inner: UnsafeCell::new(Some(value)),
        }
    }

    #[inline]
    pub fn get(&self) -> Option<&T> {
        unsafe { &*self.inner.get() }.as_ref()
    }

    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        unsafe { &mut *self.inner.get() }.as_mut()
    }

    pub fn set(&self, value: T) -> Result<(), T> {
        match self.try_insert(value) {
            Ok(_) => Ok(()),
            Err((_, value)) => Err(value),
        }
    }

    pub fn try_insert(&self, value: T) -> Result<&T, (&T, T)> {
        if let Some(old) = self.get() {
            return Err((old, value));
        }

        let slot = unsafe { &mut *self.inner.get() };
        *slot = Some(value);
        Ok(unsafe { slot.as_ref().unwrap_unchecked() })
    }
}
