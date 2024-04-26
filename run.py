from utils import log_and_run

if __name__ == "__main__":
    log_and_run([
        "cairo-run \
        --program=bootloader.json \
        --layout=recursive_with_poseidon \
        --program_input=bootloader_input.json \
        --air_public_input=bootloader_public_input.json \
        --air_private_input=bootloader_private_input.json \
        --trace_file=bootloader.trace \
        --memory_file=bootloader.memory \
        --print_output \
        --proof_mode \
        --print_info"
    ], "Running cairo1 pie in cairo0 bootloader", cwd=".")