use std::marker::Sync;
use std::cell::UnsafeCell;

/// `Internal variability` for `Lazystatic`
#[derive(Debug)]
pub struct UseAfterInit<T>(UnsafeCell<T>);
unsafe impl<T> Sync for UseAfterInit<T> {}
impl<T> UseAfterInit<T> {
    pub fn new(value: T) -> Self {
        UseAfterInit(UnsafeCell::new(value))
    }
    #[allow(unknown_lints,should_implement_trait)]
    pub fn as_ref(&self) -> &T {
        unsafe { self.0.get().as_ref().unwrap() }
    }
    // init fisrt before use it
    #[allow(unknown_lints,mut_from_ref)]
    pub fn as_mut(&self) -> &mut T {
        unsafe { self.0.get().as_mut().unwrap() }
    }
}
