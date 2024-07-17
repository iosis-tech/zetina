from builtin_selection.inner_select_builtins import inner_select_builtins
from builtin_selection.select_input_builtins import select_input_builtins
from builtin_selection.validate_builtins import validate_builtins
from common.builtin_poseidon.poseidon import PoseidonBuiltin, poseidon_hash_many
from common.cairo_builtins import HashBuiltin, EcOpBuiltin
from common.hash_chain import hash_chain
from common.bool import TRUE
from common.registers import get_ap, get_fp_and_pc
from common.signature import check_ecdsa_signature

const BOOTLOADER_VERSION = 0;

// Use an empty struct to encode an arbitrary-length array.
struct BuiltinList {
}

struct ProgramHeader {
    // The data length field specifies the length of the data (i.e., program header + program)
    // and guarantees unique decoding of the program hash.
    data_length: felt,
    bootloader_version: felt,
    program_main: felt,
    n_builtins: felt,
    // 'builtin_list' is a continuous memory segment containing the ASCII encoding of the (ordered)
    // builtins used by the program.
    builtin_list: BuiltinList,
}

struct BuiltinData {
    output: felt,
    pedersen: felt,
    range_check: felt,
    ecdsa: felt,
    bitwise: felt,
    ec_op: felt,
    keccak: felt,
    poseidon: felt,
}

// Computes the hash of a program.
// Arguments:
//  * program_data_ptr - the pointer to the program to be hashed.
//  * use_poseidon - a flag that determines whether the hashing will use Poseidon hash.
// Return values:
//  * hash - the computed program hash.
func compute_program_hash{pedersen_ptr: HashBuiltin*, poseidon_ptr: PoseidonBuiltin*}(
    program_data_ptr: felt*, use_poseidon: felt
) -> (hash: felt) {
    if (use_poseidon == 1) {
        let (hash) = poseidon_hash_many{poseidon_ptr=poseidon_ptr}(
            n=program_data_ptr[0], elements=&program_data_ptr[1]
        );
        return (hash=hash);
    } else {
        let (hash) = hash_chain{hash_ptr=pedersen_ptr}(data_ptr=program_data_ptr);
        return (hash=hash);
    }
}

// Executes a single task.
// The task is passed in the 'task' hint variable.
// Outputs of the task are prefixed by:
//   a. Output size (including this prefix)
//   b. hash_chain(ProgramHeader || task.program.data) where ProgramHeader is defined below.
// The function returns a pointer to the updated builtin pointers after executing the task.
func execute_task{builtin_ptrs: BuiltinData*, self_range_check_ptr}(
    builtin_encodings: BuiltinData*, builtin_instance_sizes: BuiltinData*, use_poseidon: felt
) {
    // Allocate memory for local variables.
    alloc_locals;

    // Get the value of fp.
    let (local __fp__, _) = get_fp_and_pc();

    // Pointer to the program data (which starts with ProgramHeader).
    local program_data_ptr: felt*;
    %{ ids.program_data_ptr = program_data_base = segments.add() %}

    // The struct of input builtin pointers pointed by the given builtin_ptrs.
    let input_builtin_ptrs: BuiltinData* = builtin_ptrs;
    local output_ptr = input_builtin_ptrs.output;

    let program_header = cast(program_data_ptr, ProgramHeader*);
    %{
        from bootloader.utils import load_program

        # Call load_program to load the program header and code to memory.
        program_address, program_data_size = load_program(
            task=task, memory=memory, program_header=ids.program_header,
            builtins_offset=ids.ProgramHeader.builtin_list)
        segments.finalize(program_data_base.segment_index, program_data_size)
    %}

    // Verify that the bootloader version is compatible with the bootloader.
    assert program_header.bootloader_version = BOOTLOADER_VERSION;

    // Call hash_chain, to verify the program hash.
    let pedersen_ptr = cast(input_builtin_ptrs.pedersen, HashBuiltin*);
    let poseidon_ptr = cast(input_builtin_ptrs.poseidon, PoseidonBuiltin*);
    with pedersen_ptr, poseidon_ptr {
        let (hash) = compute_program_hash(
            program_data_ptr=program_data_ptr, use_poseidon=use_poseidon
        );
    }

    // Write hash_chain result to output_ptr + 1.
    assert [output_ptr + 1] = hash;
    %{
        # Validate hash.
        from starkware.cairo.bootloaders.hash_program import compute_program_hash_chain

        assert memory[ids.output_ptr + 1] == compute_program_hash_chain(
            program=task.get_program(),
            use_poseidon=bool(ids.use_poseidon)), 'Computed hash does not match input.'
    %}

    local public_key: felt;
    local signature_r: felt;
    local signature_s: felt;
    %{
        ids.public_key = simple_bootloader_input.job.public_key
        ids.signature_r = simple_bootloader_input.job.signature_r
        ids.signature_s = simple_bootloader_input.job.signature_s
    %}

    let ec_op_ptr = cast(input_builtin_ptrs.ec_op, EcOpBuiltin*);
    with ec_op_ptr {
        let (res) = check_ecdsa_signature(
            message=hash, public_key=public_key, signature_r=signature_r, signature_s=signature_s
        );
        assert res = TRUE;
    }

    // Set the program entry point, so the bootloader can later run the program.
    local builtin_list: felt* = &program_header.builtin_list;
    local n_builtins = program_header.n_builtins;
    tempvar program_address = builtin_list + n_builtins;
    %{
        # Sanity check.
        assert ids.program_address == program_address
    %}
    tempvar program_main = program_header.program_main;
    // The address in memory where the main function of the task is loaded.
    local program_entry_point: felt* = program_address + program_main;

    // Fill in all builtin pointers which may be used by the task.
    // Skip the 2 slots prefix that we add to the task output.
    local pre_execution_builtin_ptrs: BuiltinData = BuiltinData(
        output=output_ptr + 2,
        pedersen=cast(pedersen_ptr, felt),
        range_check=input_builtin_ptrs.range_check,
        ecdsa=input_builtin_ptrs.ecdsa,
        bitwise=input_builtin_ptrs.bitwise,
        ec_op=cast(ec_op_ptr, felt),
        keccak=input_builtin_ptrs.keccak,
        poseidon=cast(poseidon_ptr, felt),
    );

    // Call select_input_builtins to get the relevant input builtin pointers for the task.
    select_input_builtins(
        all_encodings=builtin_encodings,
        all_ptrs=&pre_execution_builtin_ptrs,
        n_all_builtins=BuiltinData.SIZE,
        selected_encodings=builtin_list,
        n_selected_builtins=n_builtins,
    );

    call_task:
    %{
        from bootloader.objects import (
            CairoPieTask,
            Task,
        )
        from bootloader.utils import (
            load_cairo_pie,
        )

        assert isinstance(task, Task)
        n_builtins = len(task.get_program().builtins)
        new_task_locals = {}

        if isinstance(task, CairoPieTask):
            ret_pc = ids.ret_pc_label.instruction_offset_ - ids.call_task.instruction_offset_ + pc
            load_cairo_pie(
                task=task.cairo_pie, memory=memory, segments=segments,
                program_address=program_address, execution_segment_address= ap - n_builtins,
                builtin_runners=builtin_runners, ret_fp=fp, ret_pc=ret_pc)
        else:
            raise NotImplementedError(f'Unexpected task type: {type(task).__name__}.')

        vm_enter_scope(new_task_locals)
    %}

    // Call the inner program's main() function.
    call abs program_entry_point;

    ret_pc_label:
    %{
        vm_exit_scope()
        # Note that bootloader_input will only be available in the next hint.
    %}

    // Note that used_builtins_addr cannot be set in a hint because doing so will allow a malicious
    // prover to lie about the outputs of a valid program.
    let (ap_val) = get_ap();
    local used_builtins_addr: felt* = cast(ap_val - n_builtins, felt*);

    // Call inner_select_builtins to validate that the values of the builtin pointers for the next
    // task are updated according to the task return builtin pointers.

    // Allocate a struct containing all builtin pointers just after the program returns.
    local return_builtin_ptrs: BuiltinData;
    %{
        from bootloader.starknet_with_keccak.builtins import ALL_BUILTINS
        from bootloader.utils import write_return_builtins

        # Fill the values of all builtin pointers after executing the task.
        builtins = task.get_program().builtins
        write_return_builtins(
            memory=memory, return_builtins_addr=ids.return_builtin_ptrs.address_,
            used_builtins=builtins, used_builtins_addr=ids.used_builtins_addr,
            pre_execution_builtins_addr=ids.pre_execution_builtin_ptrs.address_, task=task, all_builtins=ALL_BUILTINS)

        vm_enter_scope({'n_selected_builtins': n_builtins})
    %}
    let select_builtins_ret = inner_select_builtins(
        all_encodings=builtin_encodings,
        all_ptrs=&return_builtin_ptrs,
        selected_encodings=builtin_list,
        selected_ptrs=used_builtins_addr,
        n_builtins=BuiltinData.SIZE,
    );
    %{ vm_exit_scope() %}

    // Assert that the correct number of builtins was selected.
    // Note that builtin_list is a pointer to the list containing the selected encodings.
    assert n_builtins = select_builtins_ret.selected_encodings_end - builtin_list;

    // Call validate_builtins to validate that the builtin pointers have advanced correctly.
    validate_builtins{range_check_ptr=self_range_check_ptr}(
        prev_builtin_ptrs=&pre_execution_builtin_ptrs,
        new_builtin_ptrs=&return_builtin_ptrs,
        builtin_instance_sizes=builtin_instance_sizes,
        n_builtins=BuiltinData.SIZE,
    );

    // Verify that [output_ptr] = return_builtin_ptrs.output - output_ptr.
    // Output size should be 2 + the number of output slots that were consumed by the task.
    local output_size = return_builtin_ptrs.output - output_ptr;
    assert [output_ptr] = output_size;

    %{
        from bootloader.utils import get_task_fact_topology

        # Add the fact topology of the current task to 'fact_topologies'.
        output_start = ids.pre_execution_builtin_ptrs.output
        output_end = ids.return_builtin_ptrs.output
        fact_topologies.append(get_task_fact_topology(
            output_size=output_end - output_start,
            task=task,
        ))
    %}

    let builtin_ptrs = &return_builtin_ptrs;
    return ();
}
