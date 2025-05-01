bitflags::bitflags! {
    /// Possible flags for a page table entry.
    pub struct PTEFlags: usize {
        // Attribute fields in stage 1 VMSAv8-64 Block and Page descriptors:
        /// Whether the descriptor is valid.
        const VALID =       bit!(0);
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   bit!(1);
        /// Memory attributes index field.
        const ATTR_INDX =   0b111 << 2;
        const NORMAL_NONCACHE = 0b010 << 2;
        /// Non-secure bit. For memory accesses from Secure state, specifies whether the output
        /// address is in Secure or Non-secure memory.
        const NS =          bit!(5);
        /// Access permission: accessable at EL0.
        const AP_EL0 =      bit!(6);
        /// Access permission: read-only.
        const AP_RO =       bit!(7);
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER =       bit!(8);
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   bit!(9);
        /// The Access flag.
        const AF =          bit!(10);
        /// The not global bit.
        const NG =          bit!(11);
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  bit!(52);
        /// The Privileged execute-never field.
        const PXN =         bit!(53);
        /// The Execute-never or Unprivileged execute-never field.
        const UXN =         bit!(54);

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors:

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           bit!(59);
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            bit!(60);
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     bit!(61);
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   bit!(62);
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            bit!(63);
    }
}

#[derive(Clone, Copy)]
pub struct PTE(usize);

impl PTE {
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn address(&self) -> usize {
        (self.0 & !0xFFF) as _
    }

    pub const fn set_flags(self, flags: PTEFlags) -> Self {
        Self(self.0 & !0xFFF | flags.bits())
    }

    pub const fn set_addr(self, addr: usize) -> Self {
        Self(self.0 & 0xFFF | addr)
    }

    #[inline]
    pub const fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::VALID)
    }

    #[inline]
    pub fn is_table(&self) -> bool {
        self.flags().contains(PTEFlags::NON_BLOCK | PTEFlags::VALID)
    }

    #[inline]
    pub fn new_table(paddr: usize) -> Self {
        Self(paddr | PTEFlags::VALID.bits() | PTEFlags::NON_BLOCK.bits())
    }

    #[inline]
    pub fn new_page(paddr: usize, flags: PTEFlags) -> Self {
        Self(paddr | flags.bits() as usize)
    }
}
