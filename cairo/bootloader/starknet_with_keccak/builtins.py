from starkware.cairo.lang.builtins.all_builtins import *

ALL_BUILTINS = BuiltinList(
    [
        OUTPUT_BUILTIN,
        PEDERSEN_BUILTIN,
        RANGE_CHECK_BUILTIN,
        ECDSA_BUILTIN,
        BITWISE_BUILTIN,
        EC_OP_BUILTIN,
        KECCAK_BUILTIN,
        POSEIDON_BUILTIN,
    ]
)
