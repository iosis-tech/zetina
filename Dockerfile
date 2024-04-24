# Use a Debian-based Linux distribution as the base image
FROM --platform=linux/amd64 debian:stable-slim

# Install necessary packages for Rust and Python development
RUN apt-get update && \
    apt-get install -y \
    curl \
    gcc \
    libc6-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust using Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Rust binaries to the PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Display installed Rust version
RUN rustc --version && cargo --version

# Install dependencies for Pyenv and Python
RUN apt-get update && \
    apt-get install -y \
    make \
    build-essential \
    libssl-dev \
    zlib1g-dev \
    libbz2-dev \
    libreadline-dev \
    libsqlite3-dev \
    wget \
    curl \
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
    && rm -rf /var/lib/apt/lists/*

# Install Pyenv
RUN curl https://pyenv.run | bash

RUN echo 'export PATH="/root/.pyenv/bin:$PATH"' >> /root/.bashrc && \
    echo 'eval "$(pyenv init -)"' >> /root/.bashrc && \
    echo 'eval "$(pyenv virtualenv-init -)"' >> /root/.bashrc

SHELL ["/bin/bash", "-c"]

# Set Pyenv environment variables
ENV PATH="/root/.pyenv/bin:$PATH"

# Install Python 3.9.0 using Pyenv
RUN eval "$(pyenv init -)" && \
    eval "$(pyenv virtualenv-init -)" && \
    pyenv install 3.9.0 && \
    pyenv global 3.9.0 && \
    pyenv --version && \
    python -V && \
    pip install --upgrade pip

# Install docker
RUN curl -fsSL https://get.docker.com | bash

RUN mkdir -p /root/.local/bin
RUN echo 'export PATH="/root/.local/bin:$PATH"' >> /root/.bashrc

# Set the working directory
WORKDIR /workshop

# Set the default command to run when the container starts
CMD ["bash"]
