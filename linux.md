# Linux Development Environment

Docker-based dev environment for contributors on Linux. **The recommended way to run the full desktop app on Linux is still a native install** (Rust + Tauri system deps). Docker is mainly useful for frontend dev and consistent tooling.

## Quick start

```bash
make build
make run          # shell inside container — run npm install, cargo check, etc.
make dev          # Vite only at http://localhost:1420
```

For the **full Tauri window** on Linux, install deps natively (see [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)) and run:

```bash
npm install
npm run tauri dev
```

`make tauri-dev` forwards X11 and may work with `xhost +local:` on the host, but native `npm run tauri dev` is more reliable.

## What broke in PR #1 (and fixes)

| Issue | Cause | Fix |
|-------|--------|-----|
| `npm install` / Vite crash on Linux | `package-lock.json` was regenerated on macOS, stripping `libc` metadata needed for Linux native bindings (`@rolldown/*`, `@tailwindcss/oxide`, etc.) | Restore lockfile with `libc` entries; **do not run `npm install` on macOS and commit lockfile without a Linux check** |
| `make dev` unreachable | Makefile mapped port **5173**; Vite uses **1420** | Map `1420:1420` and set `TAURI_DEV_HOST=0.0.0.0` |
| Tauri in Docker | GUI needs display + WebKit/GTK on host | Use native `tauri dev`, or X11-forwarded `make tauri-dev` |
| Node 22 in Docker only | Dockerfile pins Node 22; `package.json` `engines` and `.nvmrc` also require **Node 22+** |

## Dockerfile

Ubuntu 24.04 with Node 22, Rust (global `/opt/rust/cargo`), and Tauri build libraries (`webkit2gtk`, `libgtk-3`, etc.).

## Makefile targets

| Target | Description |
|--------|-------------|
| `make build` | Build `ai-manager-dev:latest` image |
| `make run` | Interactive bash, repo mounted at `/workspace` |
| `make dev` | Vite dev server on port 1420 |
| `make tauri-dev` | Tauri dev (needs X11 on host) |
| `make clean` | Remove local image |

## Lockfile policy

`package-lock.json` is **cross-platform**. After changing dependencies:

1. Run `npm install` on Linux (or in `make run`), or
2. Verify with `npm ci && npm run build` on Linux before merging lockfile changes from macOS.

Regenerating the lockfile only on macOS removes Linux `libc` hints and breaks installs on glibc/musl systems. Prefer:

```bash
make build
make lockfile   # npm install inside the Linux container
```

## Native Linux checklist

Requires **Node.js 22+** (`nvm install` / `nvm use` — see `.nvmrc`).

```bash
# System deps (Debian/Ubuntu) — see Tauri docs for your distro
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
nvm use    # or: fnm use / mise use
npm install
npm run tauri dev
```
