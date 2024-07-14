import os
from utils import log_and_run

if __name__ == "__main__":
    current_dir = os.getcwd()

    log_and_run([
        f"cairo-compile --cairo_path=. cairo0_fibonacci.cairo --output {current_dir}/cairo0_fibonacci_compiled.json",
    ], "Compile program", cwd=".")

    log_and_run([
        "cairo-run \
        --program=cairo0_fibonacci_compiled.json \
        --layout=recursive_with_poseidon \
        --program_input=cairo0_fibonacci_input.json \
        --cairo_pie_output=cairo0_fibonacci_pie.zip \
        --print_output \
        --print_info"
    ], "Create PIE", cwd=".")