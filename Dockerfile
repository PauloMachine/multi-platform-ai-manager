# Development container for multi-platform-ai-manager
FROM ubuntu:24.04

LABEL org.opencontainers.image.description="Development container for Tauri app"

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/home/devuser/.rustup
ENV CARGO_HOME=/home/devuser/.cargo
ENV PATH="/home/devuser/.cargo/bin:${PATH}"

# Create non-root user
RUN groupadd --gid 2000 devuser && \
    useradd --uid 2000 --gid devuser --shell /bin/bash --create-home devuser

USER root

# Install basic packages and Tauri dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        curl \
        wget \
        file \
        libssl-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        webkit2gtk-4.1-dev \
        git \
        make \
        vim \
        jq \
        htop \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js (Start with Node 18 as requested)
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs

# Install Rust
USER root
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
RUN ln -s /root/.cargo/bin/cargo /usr/local/bin/cargo && \
    ln -s /root/.cargo/bin/rustc /usr/local/bin/rustc && \
    ln -s /root/.cargo/bin/rustup /usr/local/bin/rustup

# Set up workspace
WORKDIR /workspace

# Default command
CMD ["bash"]
