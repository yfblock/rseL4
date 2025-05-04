pub trait CapTrait {
    fn raw_cap(&self) -> RawCap;
}

#[repr(C)]
#[derive(Clone)]
pub struct RawCap([usize; 2]);

impl RawCap {
    pub const fn new(val: [usize; 2]) -> Self {
        Self(val)
    }
}
