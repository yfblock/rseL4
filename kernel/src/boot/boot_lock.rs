use core::ops::Deref;

pub struct BootLock<T>(*mut T);

impl<T> BootLock<T> {
    pub const fn new(value: *mut T) -> Self {
        Self(value)
    }

    pub fn check_lock(&self) -> &'static mut T {
        // TODO: 检查当前核心是否为 BootCore
        unsafe { self.0.as_mut().unwrap() }
    }
}

impl<T> Deref for BootLock<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.0.as_ref().unwrap() }
    }
}

unsafe impl<T> Sync for BootLock<T> {}
