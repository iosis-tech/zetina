# Stage 1: Use the stone-prover image to copy the executable
FROM stone-prover AS stone-prover

# Stage 2: Use a Debian-based Linux distribution as the base image
FROM --platform=linux/amd64 debian:stable-slim

# Set the default shell to bash
SHELL ["/bin/bash", "-ci"]

# Install necessary packages for Rust and Python development
RUN apt-get update && apt-get install -y \
    curl \
    gcc \
    libc6-dev \
    make \
    build-essential \
    libssl-dev \
    zlib1g-dev \
    libbz2-dev \
    libreadline-dev \
    libsqlite3-dev \
    wget \
    llvm \
    libncurses5-dev \
    libncursesw5-dev \
    xz-utils \
    tk-dev \
    libffi-dev \
    liblzma-dev \
    python3-openssl \
    git \
    libgmp-dev \
    libdw1 \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install Rust using Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    echo 'export PATH="/root/.cargo/bin:$PATH"' >> /root/.bashrc

# Install cargo-make and cargo-nextest
RUN cargo install --force cargo-make && \
    curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin

# Install Pyenv
RUN curl https://pyenv.run | bash && \
    echo 'export PATH="/root/.pyenv/bin:$PATH"' >> /root/.bashrc && \
    echo 'eval "$(pyenv init -)"' >> /root/.bashrc && \
    echo 'eval "$(pyenv virtualenv-init -)"' >> /root/.bashrc

# Install Python 3.9.0 using Pyenv
RUN pyenv install 3.9.0 && \
    pyenv global 3.9.0 && \
    pyenv --version && \
    python -V && \
    pip install --upgrade pip

# Add Python and cargo executables to PATH
RUN mkdir -p /root/.local/bin && \
    echo 'export PATH="/root/.local/bin:$PATH"' >> /root/.bashrc

# Copy the executable from stone-prover image
COPY --from=stone-prover /bin/cpu_air_prover /root/.local/bin/
COPY --from=stone-prover /bin/cpu_air_verifier /root/.local/bin/

# Set the working directory
WORKDIR /zetina

# Copy the current directory content into the container
COPY cairo/ cairo/
COPY requirements.txt requirements.txt

# Install requirements
RUN pip install -r requirements.txt && \
    pip install cairo/

# Copy the current directory content into the container
COPY . .

# Build
RUN cargo build --release