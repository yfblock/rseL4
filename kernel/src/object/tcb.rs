use crate::arch::{ArchTCB, VirtAddr};

use super::{
    fault::{Fault, LookupFault, ThreadState},
    structures::Notification,
};

/* TCB: size >= 18 words + sizeof(arch_tcb_t) + 1 word on MCS (aligned to nearest power of 2) */
struct TCB {
    /* arch specific tcb state (including context)*/
    arch: ArchTCB,

    /* Thread state, 3 words */
    state: ThreadState,

    /* Notification that this TCB is bound to. If this is set, when this TCB waits on
     * any sync endpoint, it may receive a signal from a Notification object.
     * 1 word*/
    bound_notification: *mut Notification,

    /// Current fault, 2 words
    fault: Fault,

    /// Current lookup failure, 2 words
    lookup_failure: LookupFault,

    /// Domain, 1 byte (padded to 1 word)
    domain: usize,

    /// maximum controlled priority, 1 byte (padded to 1 word)
    mcp: usize,

    /// Priority, 1 byte (padded to 1 word)
    priority: usize,

    /// Timeslice remaining, 1 word
    time_slice: usize,

    /// Capability pointer to thread fault handler, 1 word
    fault_handler: usize,

    /// userland virtual address of thread IPC buffer, 1 word
    ipc_buffer: VirtAddr,
    // #ifdef ENABLE_SMP_SUPPORT
    //     /* cpu ID this thread is running on, 1 word */
    //     word_t tcbAffinity;
    // #endif /* ENABLE_SMP_SUPPORT */

    // /* Previous and next pointers for scheduler queues , 2 words */
    // struct tcb *tcbSchedNext;
    // struct tcb *tcbSchedPrev;
    // /* Previous and next pointers for endpoint and notification queues, 2 words */
    // struct tcb *tcbEPNext;
    // struct tcb *tcbEPPrev;
}
