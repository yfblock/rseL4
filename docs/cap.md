# Capability 相关内存和结构

```C
typedef cte_t  slot_t;
typedef cte_t *slot_ptr_t;
#define SLOT_PTR(pptr, pos) (((slot_ptr_t)(pptr)) + (pos))
#define pptr_of_cap(cap) ((pptr_t)cap_get_capPtr(cap))
```

`SLOT_PTR` 和 `CNODE` 的关系

在 `create_root_cnode` 采用这种方式来创建 CNode 的根节点。

```C
BOOT_CODE cap_t
create_root_cnode(void)
{
    cap_t cap = cap_cnode_cap_new(
                    CONFIG_ROOT_CNODE_SIZE_BITS, /* radix */
                    wordBits - CONFIG_ROOT_CNODE_SIZE_BITS, /* guard size */
                    0, /* guard */
                    rootserver.cnode); /* pptr */

    /* write the root CNode cap into the root CNode */
    write_slot(SLOT_PTR(rootserver.cnode, seL4_CapInitThreadCNode), cap);

    return cap;
}
```