# Multi-platform AI Manager

**Local-first** desktop app for monitoring usage of AI development tools (Claude Code, Cursor, Codex).

> **Important:** this app **does not have its own login**. All authentication uses each tool's official flow. No tokens, cookies, or prompts are sent to our servers.

---

## Prerequisites

| Requirement | Minimum version |
|-------------|-----------------|
| Node.js | 18+ |
| Rust | 1.70+ (install via [rustup](https://rustup.rs)) |
| macOS | 13+ (Windows 10+ also supported) |

### Provider CLIs (optional, but recommended)

For full detection, auth, and quota:

```bash
# Claude Code CLI
curl -fsSL https://claude.ai/install.sh | bash

# Cursor Agent CLI
curl https://cursor.com/install -fsS | bash

# Codex CLI (OpenAI)
npm install -g @openai/codex

# Add to PATH (if not already)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

Verify:

```bash
claude --version
cursor-agent --version   # or: agent --version
codex --version
```

---

## Running the app

```bash
npm install
npm run tauri dev        # development
npm run tauri build      # production build (.app / .dmg / .exe)
```

On first launch, the app creates `~/.multi-platform-ai-manager/` and installs hooks for detected providers.

---

## Authentication

**Multi-platform AI Manager does not ask you to log in.** It only checks whether you are already authenticated with the tools below and, if not, opens their official login flow.

### Claude Code

| Item | Details |
|------|---------|
| Account required | Claude Pro, Max, Team, Enterprise, or Console |
| Free Claude.ai plan | Does **not** include Claude Code |
| Check status | `claude auth status` |
| Log in | `claude auth login` |
| In the app | **Settings → Claude Code → Connect** |

After login, the app captures **5h/7d quota** via the statusline hook (only while Claude Code is actively in use).

### Cursor

| Item | Details |
|------|---------|
| Account required | Cursor account (individual, Team, or Enterprise) |
| Check status | `cursor-agent status` |
| Log in | `cursor-agent login` |
| In the app | **Settings → Cursor → Connect** |

Login opens the browser at `cursor.com/loginDeepControl`. After authentication, the CLI stays logged in locally — the app **does not store** tokens.

### Codex

| Item | Details |
|------|---------|
| Account required | OpenAI account with Codex access |
| Check status | `codex login status` |
| Log in | `codex login` |
| In the app | **Settings → Codex → Connect** |

Login opens OpenAI's official flow. After authentication, the CLI stays logged in locally — the app **does not store** tokens.

### What works without login

| Provider | Without login |
|----------|---------------|
| Claude Code | Detects installation, but shows "Not connected" and no quota |
| Cursor | May detect the installed app; session hooks work if Cursor is open |
| Codex | Detects installation, but shows "Not connected" |

### What works after login

| Provider | With login |
|----------|------------|
| Claude Code | 5h/7d quota (via statusline), sessions, models, history |
| Cursor | "Connected" status, sessions, models, real-time activity |
| Codex | "Connected" status, sessions, models, real-time activity |

> **Cursor quota (individual):** there is no official API for usage percentage. The dashboard shows **"Usage limit unavailable"** — this is expected. Sessions and models are still monitored via hooks.

> **Codex quota:** available only in the TUI (`/usage`). There is no official JSON export via CLI — the dashboard shows **"Usage limit unavailable"**, but sessions and models are monitored via hooks.

---

## Quick setup (first time)

1. Install and run the app (`npm run tauri dev`)
2. Complete onboarding (detect providers)
3. Authenticate each tool:

```bash
claude auth login
cursor-agent login
codex login
```

4. Or use **Settings → Connect** in the app
5. Click **Refresh** on the dashboard

Hooks are installed automatically in:

- `~/.claude/settings.json` (Claude Code + statusline)
- `~/.cursor/hooks.json` (Cursor)
- `~/.codex/hooks.json` (Codex)

Previous configs are backed up as `*.bak-multi-platform-ai-manager`.

---

## Local data

Everything stays in `~/.multi-platform-ai-manager/`:

| File | Contents |
|------|----------|
| `config.json` | Preferences (refresh interval, notifications, etc.) |
| `events.ndjson` | Normalized event history |
| `sessions.json` | Session cache |
| `claude-usage.json` | Claude quota (updated by statusline hook) |
| `hooks/` | Hook scripts installed by the app |
| `handoff/` | Session continuation prompts and launch scripts (`run-{sessionId}.txt`, `launch-session.sh`) |

---

## Copy session (continue on another platform)

When a session hits a limit on one platform — or you simply want to switch tools — you can **continue the same work on another provider** directly from **Sessions**.

### How to use

1. Open **Sessions** in the sidebar (or open a session from the dashboard)
2. Click **Copy session** on the session row or on the session detail page
3. Choose the **target platform** (only providers different from the current one are shown)
4. The app opens a **terminal** in the session's project directory and starts the target CLI with a continuation prompt

### What the prompt includes

| Field | Source |
|-------|--------|
| Original task | `initial_prompt` from the session |
| Work already done | Activity log (tools run, files read/edited, etc.) |
| Tools used | Aggregated from hook events |
| Files modified | From `afterFileEdit` hooks |
| Transcript excerpt | Optional, when a transcript path is available |

### CLIs launched per platform

| Platform | CLI |
|----------|-----|
| Claude Code | `claude` |
| Cursor | `cursor-agent` (or `agent`) |
| Codex | `codex` |

The target CLI must be **installed and on PATH**. If it is missing, the app shows an error on the button.

### Saved files

| Path | Purpose |
|------|---------|
| `~/.multi-platform-ai-manager/handoff/run-{sessionId}.txt` | Continuation prompt for the selected session |
| `~/.multi-platform-ai-manager/handoff/launch-session.sh` | Shell script (macOS / Linux) |
| `%USERPROFILE%\.multi-platform-ai-manager\handoff\launch-session.cmd` | Batch script (Windows) |

> **Note:** the app launches the provider's **official CLI** in your system terminal — **Terminal.app** on macOS, **Command Prompt** on Windows. It does not embed a terminal or automate APIs beyond what each CLI supports natively.

---

## Build and validation

```bash
npm run lint              # TypeScript check
npm run build             # frontend
cd tauri && cargo check
npm run tauri build       # packaged app
```

---

## Known limitations

- **Claude quota:** only updates while Claude Code is running (statusline hook); there is no `claude usage --json`
- **Cursor quota:** unavailable for individual accounts via official API
- **Codex quota:** unavailable via CLI (TUI `/usage` only); sessions via hooks
- **Authentication:** done once per CLI; the app only triggers the official flow and never stores credentials
- **Copy session:** requires the target provider CLI to be installed; opens the system terminal (macOS: Terminal.app, Windows: Command Prompt via `cmd`)
