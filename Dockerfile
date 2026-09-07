# Development container for multi-platform-ai-manager
FROM ubuntu:24.04

LABEL org.opencontainers.image.description="Development container for Tauri app"

# Set environment variables
ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/opt/rust/rustup
ENV CARGO_HOME=/opt/rust/cargo
ENV PATH="/opt/rust/cargo/bin:${PATH}"

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

# Install Node.js (Upgrade to LTS Node 22)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - && \
    apt-get install -y nodejs

# Install Rust globally
RUN mkdir -p /opt/rust/rustup /opt/rust/cargo && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path && \
    chmod -R a+rwX /opt/rust

# Set up workspace
WORKDIR /workspace

# Default command
CMD ["bash"]
