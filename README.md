<h1 align="center">tmux-agent-sidebar</h1>

<p align="center">A tmux TUI that tracks every Claude Code, Codex, and OpenCode pane across every session and window. See status, background shells, prompts, Git state, activity, and worktrees in a floating window.</p>

<p align="center">
  <img src="docs/screenshots/full-screen-preview.png" alt="tmux-agent-sidebar full-screen preview" width="800" />
</p>

---

## 🚀 Floating Window TUI

This fork transforms the persistent sidebar into a **disposable floating window**. Toggle it instantly when you need to check agent status, then dismiss it to keep your workspace clean.

<p align="center">
  <img src="docs/screenshots/floating-popup.png" alt="tmux-agent-sidebar floating popup" width="800" />
</p>

### Features

- **Global Tracking** — Monitors all AI agents across all tmux sessions and windows.
- **Rich Metadata** — Prompts, tool calls, background shell state, and task progress.
- **Git & Worktrees** — Manage worktrees and view Git state directly from the TUI.
- **Minimal Impact** — Runs as a `display-popup` process; no persistent pane required.

---

## 🔔 Window Bell Notifications

Stay informed without keeping the TUI open. The plugin integrates with tmux's window bell to alert you when an agent requires attention or finishes a task.

<p align="center">
  <img src="docs/screenshots/bell-notification.png" alt="tmux window bell notification" width="600" />
</p>

---

## 🛠️ Usage

Bind a key to toggle the floating TUI in your `tmux.conf`:

```tmux
# Toggle floating sidebar (90% width/height)
bind e display-popup -EE -w 90% -h 90% "tmux-agent-sidebar"
```

---

## 📦 Installation & Setup

This repository is a fork with UI improvements. For detailed installation instructions, setup wizards, and agent hook configuration, please refer to the **[upstream repository](https://github.com/hiroppy/tmux-agent-sidebar)**.

---

## License

[MIT](./LICENSE)
