--
-- Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
--
-- SPDX-License-Identifier: GPL-2.0-only
--

block null_cap {
    padding 64

    field capType 5
    padding 59
}

block untyped_cap {
#if BF_CANONICAL_RANGE == 48
    field capFreeIndex 48
    padding 9
#elif BF_CANONICAL_RANGE == 39
    field capFreeIndex 39
    padding 18
#else
#error "Unspecified canonical address range"
#endif
    field capIsDevice 1
    field capBlockSize 6

    field capType 5
#if BF_CANONICAL_RANGE == 48
    padding 11
    field_high capPtr 48
#elif BF_CANONICAL_RANGE == 39
    padding 20
    field_high capPtr 39
#else
#error "Unspecified canonical address range"
#endif
}

block endpoint_cap(capEPBadge, capCanGrantReply, capCanGrant, capCanSend,
                   capCanReceive, capEPPtr, capType) {
    field capEPBadge 64

    field capType 5
    field capCanGrantReply 1
    field capCanGrant 1
    field capCanReceive 1
    field capCanSend 1
#if BF_CANONICAL_RANGE == 48
    padding 7
    field_high capEPPtr 48
#elif BF_CANONICAL_RANGE == 39
    padding 16
    field_high capEPPtr 39
#else
#error "Unspecified canonical address range"
#endif

}

block notification_cap {
    field capNtfnBadge 64

    field capType 5
    field capNtfnCanReceive 1
    field capNtfnCanSend 1
#if BF_CANONICAL_RANGE == 48
    padding 9
    field_high capNtfnPtr 48
#elif BF_CANONICAL_RANGE == 39
    padding 18
    field_high capNtfnPtr 39
#else
#error "Unspecified canonical address range"
#endif
}

#ifdef CONFIG_KERNEL_MCS
block reply_cap {
    field capReplyPtr 64

    field capType 5
    field capReplyCanGrant 1
    padding 58
}

block call_stack(callStackPtr, isHead) {
    padding 15
    field isHead 1
#if BF_CANONICAL_RANGE == 48
    field_high callStackPtr 48
#elif BF_CANONICAL_RANGE == 39
	padding 9
    field_high callStackPtr 39
#else
#error "Unspecified canonical address range"
#endif
}
#else
block reply_cap(capReplyCanGrant, capReplyMaster, capTCBPtr, capType) {
    field capTCBPtr 64

    field capType 5
    padding 57
    field capReplyCanGrant 1
    field capReplyMaster 1
}
#endif

-- The user-visible format of the data word is defined by cnode_capdata, below.
block cnode_cap(capCNodeRadix, capCNodeGuardSize, capCNodeGuard,
                capCNodePtr, capType) {
    field capCNodeGuard 64

    field capType 5
    field capCNodeGuardSize 6
    field capCNodeRadix 6
#if BF_CANONICAL_RANGE == 48
    field_high capCNodePtr 47
#elif BF_CANONICAL_RANGE == 39
    padding 9
    field_high capCNodePtr 38
#else
#error "Unspecified canonical address range"
#endif
}

block thread_cap {
    padding 64

    field capType 5
#if BF_CANONICAL_RANGE == 48
    padding 11
    field_high capTCBPtr 48
#elif BF_CANONICAL_RANGE == 39
    padding 20
    field_high capTCBPtr 39
#else
#error "Unspecified canonical address range"
#endif
}

block irq_control_cap {
    padding 64

    field capType  5
    padding 59
}

block irq_handler_cap {
#ifdef ENABLE_SMP_SUPPORT
    field capIRQ 64
#else
    padding 52
    field capIRQ 12
#endif

    field capType  5
    padding 59
}

block zombie_cap {
    field capZombieID     64

    field capType         5
    padding               52
    field capZombieType   7
}

block domain_cap {
    padding 64

    field capType 5
    padding 59
}

#ifdef CONFIG_KERNEL_MCS
block sched_context_cap {
#if BF_CANONICAL_RANGE == 48
    field_high capSCPtr 48
#elif BF_CANONICAL_RANGE == 39
    padding 9
    field_high capSCPtr 39
#else
#error "Unspecified canonical address range"
#endif
    field capSCSizeBits 6
    padding 10

    field capType 5
    padding       59
}

block sched_control_cap {
    field core    64

    field capType 5
    padding       59
}
#endif

---- Arch-independent object types

-- Endpoint: size = 16 bytes
block endpoint {
    field epQueue_head 64

#if BF_CANONICAL_RANGE == 48
    padding 16
    field_high epQueue_tail 46
#elif BF_CANONICAL_RANGE == 39
    padding 25
    field_high epQueue_tail 37
#else
#error "Unspecified canonical address range"
#endif
    field state 2
}

-- Async endpoint: size = 32 bytes (64 bytes on mcs)
block notification {
#if BF_CANONICAL_RANGE == 48
#ifdef CONFIG_KERNEL_MCS
    padding 192
    padding 16
    field_high ntfnSchedContext 48
#endif
    padding 16
    field_high ntfnBoundTCB 48
#elif BF_CANONICAL_RANGE == 39
#ifdef CONFIG_KERNEL_MCS
    padding 192
    padding 25
    field_high ntfnSchedContext 39
#endif
    padding 25
    field_high ntfnBoundTCB 39
#else
#error "Unspecified canonical address range"
#endif

    field ntfnMsgIdentifier 64

#if BF_CANONICAL_RANGE == 48
    padding 16
    field_high ntfnQueue_head 48
#elif BF_CANONICAL_RANGE == 39
    padding 25
    field_high ntfnQueue_head 39
#else
#error "Unspecified canonical address range"
#endif

#if BF_CANONICAL_RANGE == 48
    field_high ntfnQueue_tail 48
    padding 14
#elif BF_CANONICAL_RANGE == 39
    field_high ntfnQueue_tail 39
    padding 23
#else
#error "Unspecified canonical address range"
#endif
    field state 2
}
