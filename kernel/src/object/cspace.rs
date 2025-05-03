use crate::{arch::VirtAddr, boot::consts::RootCNodeCapSlots};

use super::{
    cap::{CapTrait, RawCap},
    structures::DomainCap,
    MDBNode,
};

#[derive(Default)]
pub struct CNode(VirtAddr);

impl CNode {
    /// 创建一个 [CNode] 结构体
    ///
    /// ## 参数
    /// - `addr` [VirtAddr] CNode 指向的物理内存
    pub const fn new(addr: VirtAddr) -> Self {
        Self(addr)
    }

    /// 获取 [CNode] 指向的内存地址 [VirtAddr]
    pub const fn cnode_addr(&self) -> VirtAddr {
        self.0
    }

    /// 在 `offset` 出写入一个 Capability
    ///
    /// ## 参数
    /// - `offset` [usize] 需要写入的 Capability 偏移
    /// - `cap`    [CapTrait] 需要写入的 Capability，在内部会转换为 [RawCap] 后写入
    pub fn write(&mut self, offset: usize, cap: impl CapTrait) {
        let mut mdb_node = MDBNode::new(cap.raw_cap());
        mdb_node.set_first_badged(true);
        mdb_node.set_revocable(true);
        unsafe {
            self.0
                .as_mut_ptr::<MDBNode<RawCap>>()
                .add(offset)
                .write(mdb_node);
        }
    }

    pub fn create_domain_cap(&mut self) {
        let cap = DomainCap::empty();
        self.write(RootCNodeCapSlots::Domain as _, cap);
    }
}
