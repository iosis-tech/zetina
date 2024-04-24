import subprocess

def log_and_run(commands, description, cwd=None):
    from colorama import Fore, Style
    
    full_command = " && ".join(commands)
    try:
        print(f"{Fore.YELLOW}Starting: {description}...{Style.RESET_ALL}")
        print(f"{Fore.CYAN}Command: {full_command}{Style.RESET_ALL}")
        result = subprocess.run(
            full_command, shell=True, check=True, cwd=cwd, text=True
        )
        print(f"{Fore.GREEN}Success: {description} completed!\n{Style.RESET_ALL}")
    except subprocess.CalledProcessError as e:
        print(
            f"{Fore.RED}Error running command '{full_command}': {e}\n{Style.RESET_ALL}"
        )


if __name__ == "__main__":
    subprocess.run(["pip", "install", "-r", "requirements.txt"], check=True)

    log_and_run(["pip install cairo/"], "Install bootloader package", cwd=".")

    log_and_run(
        [
            "git clone https://github.com/starkware-libs/stone-prover.git",
        ],
        "Clone stone-prover",
        cwd=".",
    )

    log_and_run(
        [
            "docker build --tag prover .",
            "container_id=$(docker create prover)",
            "docker cp -L ${container_id}:/bin/cpu_air_prover $HOME/.local/bin",
            "docker cp -L ${container_id}:/bin/cpu_air_verifier $HOME/.local/bin",
        ],
        "Build & Install stone-prover",
        cwd="stone-prover",
    )
