--
-- Copyright 2020, Data61, CSIRO (ABN 41 687 119 230)
--
-- SPDX-License-Identifier: GPL-2.0-only
--

#include <config.h>
-- Default base size: uint64_t
#define BF_CANONICAL_RANGE 48

-- Including the common structures.bf is necessary because
-- we need the structures to be visible here when building
-- the capType
#include <structures.bf>
 
---- ARM-specific caps

block frame_cap {
    field capFMappedASID             16
    field_high capFBasePtr           48

    field capType                    5
    field capFSize                   2
    field_high capFMappedAddress     48
    field capFVMRights               2
    field capFIsDevice               1
    padding                          6
}

-- Page table caps
block page_table_cap {
    field capPTMappedASID            16
    field_high capPTBasePtr          48

    field capType                    5
    padding                          10
    field capPTIsMapped              1
    field_high capPTMappedAddress    28
    padding                          20
}

-- First-level page table (vspace_root)
block vspace_cap {
    field capVSMappedASID            16
    field_high capVSBasePtr          48

    field capType                    5
    field capVSIsMapped              1
#ifdef CONFIG_ARM_SMMU
    field capVSMappedCB              8
    padding                          50
#else
    padding                          58
#endif
}

-- Cap to the table of 2^7 ASID pools
block asid_control_cap {
    padding                          64

    field capType                    5
    padding                          59
}

-- Cap to a pool of 2^9 ASIDs
block asid_pool_cap {
    padding                         64

    field capType                   5
    field capASIDBase               16
    padding                         6
    field_high capASIDPool          37
}


#ifdef CONFIG_ARM_SMMU

block sid_control_cap {
    padding        64

    field capType  5
    padding        59
}

block sid_cap {

    padding              52
    field capSID         12

    field capType        5
    padding 59
}

block cb_control_cap {
    padding              64

    field capType        5
    padding              59
}


block cb_cap {

    padding               44
    field capBindSID      12
    field capCB           8


    field capType         5
    padding               59
}

#endif


-- NB: odd numbers are arch caps (see isArchCap())
tagged_union cap capType {
    -- 5-bit tag caps
    tag null_cap                    0
    tag untyped_cap                 2
    tag endpoint_cap                4
    tag notification_cap            6
    tag reply_cap                   8
    tag cnode_cap                   10
    tag thread_cap                  12
    tag irq_control_cap             14
    tag irq_handler_cap             16
    tag zombie_cap                  18
    tag domain_cap                  20
#ifdef CONFIG_KERNEL_MCS
    tag sched_context_cap           22
    tag sched_control_cap           24
#endif

    -- 5-bit tag arch caps
    tag frame_cap                   1
    tag page_table_cap              3
    tag vspace_cap                  9
    tag asid_control_cap            11
    tag asid_pool_cap               13

#ifdef CONFIG_ARM_SMMU
    tag sid_control_cap             17
    tag sid_cap                     19
    tag cb_control_cap              21
    tag cb_cap                      23
#endif
#ifdef CONFIG_ALLOW_SMC_CALLS
    tag smc_cap                     25
#endif
}

---- Arch-independent object types

block VMFault {
    field address                   64
    field FSR                       32
    field instructionFault          1
    padding                         27
    field seL4_FaultType            4
}

-- VM attributes

block vm_attributes {
    padding                         61
    field armExecuteNever           1
    field armParityEnabled          1
    field armPageCacheable          1
}

---- ARM-specific object types

block asid_map_none {
    padding                         63
    field type                      1
}

--- hw_vmids are required in hyp mode
block asid_map_vspace {
#ifdef CONFIG_ARM_SMMU
    field bind_cb                   8
    padding                         8
#else
    padding                         16
#endif
    field_high vspace_root          36
    padding                         11
    field type                      1
}

tagged_union asid_map type {
    tag asid_map_none 0
    tag asid_map_vspace 1
}

#ifdef CONFIG_HARDWARE_DEBUG_API

block dbg_bcr {
    padding 34
    padding 1
    padding 5
    field breakpointType 4
    field lbn 4
    field ssc 2
    field hmc 1
    padding 4
    field bas 4
    padding 2
    field pmc 2
    field enabled 1
}

block dbg_wcr {
    padding 34
    padding 1
    field addressMask 5
    padding 3
    field watchpointType 1
    field lbn 4
    field ssc 2
    field hmc 1
    field bas 8
    field lsc 2
    field pac 2
    field enabled 1
}

#endif /* CONFIG_HARDWARE_DEBUG_API */

#include <sel4/arch/shared_types.bf>
