pub mod structures;

use core::ops::{Deref, DerefMut};

/// Mapping Database Node
///
/// seL4 中的定义结构如下：
///
/// ```plain
/// -- Mapping database (MDB) node: size = 16 bytes
/// block mdb_node {
///     #if BF_CANONICAL_RANGE == 48
///         padding 16
///         field_high mdbNext 46
///     #elif BF_CANONICAL_RANGE == 39
///         padding 25
///         field_high mdbNext 37
///     #else
///     #error "Unspecified canonical address range"
///     #endif
///         field mdbRevocable 1
///         field mdbFirstBadged 1
///     
///         field mdbPrev 64
/// }    
/// ```
pub struct MDBNode<T> {
    value: T,
    next: usize,
    prev: usize,
}

impl<T> MDBNode<T> {
    pub const fn next(&self) -> usize {
        self.next & !0x3
    }
    pub const fn set_next(&mut self, next: usize) {
        self.next = self.next & 0x3 | next
    }
    pub const fn set_first_badged(&mut self, first_badge: bool) {
        self.next = self.next & !0x2 | (first_badge as usize);
    }
    pub const fn first_badge(&self) -> bool {
        self.next & 0x2 != 0
    }
    pub const fn set_revocable(&mut self, revocable: bool) {
        self.next = self.next & !0x1 | (revocable as usize);
    }
    pub const fn revocable(&self) -> bool {
        self.next & 0x1 != 0
    }
    pub const fn prev(&self) -> usize {
        self.prev
    }
    pub const fn set_prev(&mut self, prev: usize) {
        self.prev = prev;
    }
}

impl<T> Deref for MDBNode<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for MDBNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
