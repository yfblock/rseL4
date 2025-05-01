macro_rules! include_defines {
    () => {
        r"
        .ifndef MACRO_INCLUDE_FLAG
        .equ    MACRO_INCLUDE_FLAG, 1

        .equ    DAIFSET_MASK,       0xf

        .equ    ESR_EC_SHIFT,       26
        .equ    ESR_EC_LEL_DABT,    0x24
        .equ    ESR_EC_CEL_DABT,    0x25
        .equ    ESR_EC_LEL_IABT,    0x20
        .equ    ESR_EC_CEL_IABT,    0x21
        .equ    ESR_EC_LEL_SVC64,   0x15
        .equ    ESR_EL1_EC_ENFP,    0x7

        .macro  END_FUNC name
            .size  \name, .-\name
        .endm

        .macro  BEGIN_FUNC name
            .global \name
            \name:
        .endm

        .endif"
    };
}
