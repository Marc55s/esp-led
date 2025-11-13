# Use the latest Debian base image
FROM debian:latest

# Set non-interactive frontend for apt-get to avoid prompts
ENV DEBIAN_FRONTEND=noninteractive

# Update package lists and install all requested dependencies in a single layer
RUN apt-get update && \
    apt-get install -y \
    curl \
    sudo \
    git \
    wget \
    flex \
    bison \
    gperf \
    python3 \
    python3-pip \
    python3-venv \
    cmake \
    ninja-build \
    ccache \
    libffi-dev \
    libssl-dev \
    dfu-util \
    libusb-1.0-0 && \
    # Clean up the apt cache to reduce the final image size
    rm -rf /var/lib/apt/lists/*

# Install rustup (the recommended Rust installer for the latest version)
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

RUN cargo install espup
RUN cargo install ldproxy
RUN espup install
RUN . /root/export-esp.sh

# Set a working directory (optional, but good practice)
WORKDIR /app

# Define a mount point for your Rust project
# At runtime, you can mount your local folder (e.g., "my_project") like this:
# docker run -v ./my_project:/app/rust[image_name]
VOLUME /app/rust

# Define a default command to run when the container starts (e.g., a shell)
CMD ["/bin/bash"]
