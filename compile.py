import os
from utils import log_and_run

if __name__ == "__main__":
    current_dir = os.getcwd()

    log_and_run([
        f"cairo-compile --cairo_path=. bootloader/starknet/simple_bootloader.cairo --output {current_dir}/bootloader.json --proof_mode",
    ], "Compile bootloader program", cwd="cairo")