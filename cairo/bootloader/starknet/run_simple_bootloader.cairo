from bootloader.starknet.execute_task import BuiltinData, execute_task
from common.cairo_builtins import HashBuiltin, PoseidonBuiltin
from common.registers import get_fp_and_pc

// Loads the programs and executes them.
//
// Hint Arguments:
// simple_bootloader_input - contains the tasks to execute.
//
// Returns:
// Updated builtin pointers after executing all programs.
// fact_topologies - that corresponds to the tasks (hint variable).
func run_simple_bootloader{
    output_ptr: felt*,
    pedersen_ptr: HashBuiltin*,
    range_check_ptr,
    ecdsa_ptr,
    bitwise_ptr,
    ec_op_ptr,
    poseidon_ptr: PoseidonBuiltin*,
}() {
    alloc_locals;
    local task_range_check_ptr;

    %{
        # Task range checks are located right after simple bootloader validation range checks, and
        # this is validated later in this function.
        ids.task_range_check_ptr = ids.range_check_ptr + ids.BuiltinData.SIZE 
        # A list of fact_toplogies that instruct how to generate the fact from the program output
        # for each task.
        fact_topologies = []
    %}

    // A struct containing the pointer to each builtin.
    local builtin_ptrs_before: BuiltinData = BuiltinData(
        output=cast(output_ptr, felt),
        pedersen=cast(pedersen_ptr, felt),
        range_check=task_range_check_ptr,
        ecdsa=ecdsa_ptr,
        bitwise=bitwise_ptr,
        ec_op=ec_op_ptr,
        poseidon=cast(poseidon_ptr, felt),
    );

    // A struct containing the encoding of each builtin.
    local builtin_encodings: BuiltinData = BuiltinData(
        output='output',
        pedersen='pedersen',
        range_check='range_check',
        ecdsa='ecdsa',
        bitwise='bitwise',
        ec_op='ec_op',
        poseidon='poseidon',
    );

    local builtin_instance_sizes: BuiltinData = BuiltinData(
        output=1, pedersen=3, range_check=1, ecdsa=2, bitwise=5, ec_op=7, poseidon=6
    );

    // Call execute_tasks.
    let (__fp__, _) = get_fp_and_pc();

    let builtin_ptrs = &builtin_ptrs_before;
    let self_range_check_ptr = range_check_ptr;
    with builtin_ptrs, self_range_check_ptr {
        execute(
            builtin_encodings=&builtin_encodings, builtin_instance_sizes=&builtin_instance_sizes
        );
    }

    // Verify that the task range checks appear after the self range checks of execute_task.
    assert self_range_check_ptr = task_range_check_ptr;

    // Return the updated builtin pointers.
    local builtin_ptrs: BuiltinData* = builtin_ptrs;
    let output_ptr = cast(builtin_ptrs.output, felt*);
    let pedersen_ptr = cast(builtin_ptrs.pedersen, HashBuiltin*);
    let range_check_ptr = builtin_ptrs.range_check;
    let ecdsa_ptr = builtin_ptrs.ecdsa;
    let bitwise_ptr = builtin_ptrs.bitwise;
    let ec_op_ptr = builtin_ptrs.ec_op;
    let poseidon_ptr = cast(builtin_ptrs.poseidon, PoseidonBuiltin*);

    // 'execute_tasks' runs untrusted code and uses the range_check builtin to verify that
    // the builtin pointers were advanced correctly by said code.
    // Since range_check itself is used for the verification, we cannot assume that the verification
    // above is sound unless we know that the self range checks that were used during verification
    // are indeed valid (that is, within the segment of the range_check builtin).
    // Following the Cairo calling convention, we can guarantee the validity of the self range
    // checks by making sure that range_check_ptr >= self_range_check_ptr.
    // The following check validates that the inequality above holds without using the range check
    // builtin.
    let additional_range_checks = range_check_ptr - self_range_check_ptr;
    verify_non_negative(num=additional_range_checks, n_bits=64);

    return ();
}

// Verifies that a field element is in the range [0, 2^n_bits), without relying on the range_check
// builtin.
func verify_non_negative(num: felt, n_bits: felt) {
    if (n_bits == 0) {
        assert num = 0;
        return ();
    }

    tempvar num_div2 = nondet %{ ids.num // 2 %};
    tempvar bit = num - (num_div2 + num_div2);
    // Check that bit is 0 or 1.
    assert bit = bit * bit;
    return verify_non_negative(num=num_div2, n_bits=n_bits - 1);
}

// Executes the last n_tasks from simple_bootloader_input.tasks.
//
// Arguments:
// builtin_encodings - String encodings of the builtins.
// builtin_instance_sizes - Mapping to builtin sizes.
// n_tasks - The number of tasks to execute.
//
// Implicit arguments:
// builtin_ptrs - Pointer to the builtin pointers before/after executing the tasks.
// self_range_check_ptr - range_check pointer (used for validating the builtins).
//
// Hint arguments:
// job - A job to execute.
func execute{builtin_ptrs: BuiltinData*, self_range_check_ptr}(
    builtin_encodings: BuiltinData*, builtin_instance_sizes: BuiltinData*
) {
    // Allocate memory for local variables.
    alloc_locals;

    // Get the value of fp.
    let (local __fp__, _) = get_fp_and_pc();

    %{
        from bootloader.objects import Task

        # Pass current task to execute_task.
        task = simple_bootloader_input.job.load_task()
    %}
    tempvar use_poseidon = nondet %{ 1 if task.use_poseidon else 0 %};
    // Call execute_task to execute the current task.
    execute_task(
        builtin_encodings=builtin_encodings,
        builtin_instance_sizes=builtin_instance_sizes,
        use_poseidon=use_poseidon,
    );

    local executor: felt;
    local delegator: felt;

    %{
        ids.executor = simple_bootloader_input.public_key
        ids.delegator = simple_bootloader_input.job.public_key
    %}

    assert [builtin_ptrs.output + 0] = executor;
    assert [builtin_ptrs.output + 1] = delegator;

    local return_builtin_ptrs: BuiltinData = BuiltinData(
        output=builtin_ptrs.output + 2,
        pedersen=builtin_ptrs.pedersen,
        range_check=builtin_ptrs.range_check,
        ecdsa=builtin_ptrs.ecdsa,
        bitwise=builtin_ptrs.bitwise,
        ec_op=builtin_ptrs.ec_op,
        poseidon=builtin_ptrs.poseidon,
    );

    let builtin_ptrs = &return_builtin_ptrs;

    return ();
}
