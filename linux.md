# Linux Development Environment Plan

This document outlines the plan to set up a Docker-based development environment for the `multi-platform-ai-manager` repository, mimicking the workflow used in `llm-sandbox`.

## 1. Development Environment Setup

### Dockerfile
We will create a `Dockerfile` based on Ubuntu that provides all necessary tools for Tauri development.

**Required Tools:**
- **Node.js**: Start with version 18 (as currently used), then upgrade to LTS.
- **Rust**: Install the latest stable Rust toolchain via `rustup`.
- **Tauri Dependencies**:
    - `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `webkit2gtk-4.1-dev`.
- **Utilities**: `git`, `make`, `vim`, `jq`, `htop`.
- **User**: Create a non-root user `devuser` (UID/GID 2000) to avoid permission issues with host mounts.

### Makefile
We will create a `Makefile` to encapsulate common Docker commands.

**Targets:**
- `make build`: Builds the development image.
- `make run`: Launches an interactive bash shell in the container with the current directory mounted.
- `make dev`: Runs `npm run dev` (Vite) inside the container.
- `make tauri-dev`: Runs `npm run tauri dev` inside the container.
- `make clean`: Removes the local Docker image.

## 2. Verification & Iteration

1. **Build & Launch**:
    - Execute `make build`.
    - Execute `make run` and verify `node`, `npm`, `rustc`, and `cargo` are available.
2. **App Startup**:
    - Attempt to run the frontend dev server (`make dev`).
    - Verify the build process for Tauri works within the container.

## 3. Node.js LTS Upgrade

Once the basic environment is verified:
1. Update the `Dockerfile` to use the latest supported Node.js LTS version (e.g., Node 20 or 22).
2. Rebuild the image (`make build`).
3. Verify that the application still builds and runs correctly.
4. Update `package.json` or `.nvmrc` if applicable to reflect the new version.

## 4. Delivery

- All changes will be submitted via a draft Pull Request.
- The PR will include the `linux.md` spec, `Dockerfile`, `Makefile`, and the Node.js upgrade.

## Action Items
- [ ] Create `linux.md` (Done)
- [ ] Create draft PR with `linux.md`
- [ ] Implement `Dockerfile`
- [ ] Implement `Makefile`
- [ ] Verify `make build` and `make run`
- [ ] Verify app starts on Linux (within container)
- [ ] Upgrade Node.js to LTS
- [ ] Finalize and submit PR
