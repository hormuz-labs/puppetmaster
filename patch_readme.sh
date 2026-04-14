cat << 'INNER_EOF' > README.md
# OpenCode Telegram Interface

[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-blue.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Teloxide Framework](https://img.shields.io/badge/framework-teloxide--0.17-black.svg?style=flat-square)](https://github.com/teloxide/teloxide)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)
[![Status](https://img.shields.io/badge/status-active-success.svg?style=flat-square)](#)

> A robust, high-performance Telegram bot acting as a remote control and conversational interface for the [OpenCode AI platform](https://github.com/alpha-innovation-labs/opencode-rs-sdk).

This repository provides a seamless integration between Telegram and a locally running OpenCode daemon. It leverages Rust's asynchronous ecosystem to deliver live AI streaming responses, sophisticated state management for multiple concurrent users, and interactive UI elements natively within the Telegram client.

⭐️ **If you found this repository useful, please consider giving it a star!** ⭐️

---

## Table of Contents
- [Features](#features)
- [Architecture](#architecture)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Usage](#usage)

---

## Features

### Real-Time AI Streaming & Thinking
Uses Server-Sent Events (SSE) to capture output deltas from the OpenCode LLM in real-time. It elegantly handles the AI's "Thinking" process inside a blockquote and responds with real-time text chunks. 

### Multi-Modal Inputs (Voice, Images, Files)
- **Voice Memos**: Send a voice message! The bot downloads it, transcribes it via Google Cloud Speech-to-Text, and sends it directly to the AI.
- **Images & Documents**: Upload screenshots, photos, or code files. The bot reads them and attaches them natively to the prompt.

### Direct Bash Execution
Prefix any message with `!` to instantly run a bash command on the host machine within the context of your current project (e.g. `!ls -la` or `!cargo build`). STDOUT and STDERR are streamed back natively as formatted code blocks!

### Fetch Local Files
Use the `/fetch <path>` command to quickly grab files from your host machine (either relative to the active project directory or an absolute path) and receive them in Telegram as documents or images.

### Intelligent Payload Handling
Bypasses Telegram's strict 4096-character message limits natively. Long responses (such as complete code file generation or huge command outputs) are dynamically split into discrete, sequential message bubbles on-the-fly.

### Safe Markdown/HTML Parsing
Employs `pulldown-cmark` and a custom HTML renderer designed specifically for Telegram. It handles LaTeX math blocks gracefully via `unicodeit`, nested formatting, and code blocks perfectly.

### Persistent Context & State Management
Powered by `teloxide`'s `dptree` macro architecture, the bot maintains isolated `InMemStorage` state tracking for an arbitrary number of users. 
- **Session Continuity:** Users can drop out and return later while maintaining their active AI session.
- **Directory Context:** Allows switching the active project context directly from the chat.
- **Model Agnostic:** Supports hot-swapping between >100+ LLM models provisioned by OpenCode.

### Native Interactive UI
Automatically discovers connected AI providers and recent project worktrees directly from the OpenCode REST API, converting them into persistent, native Telegram Keyboard markups.

---

## Architecture

| Component | Responsibility | Technology |
| :--- | :--- | :--- |
| **Telegram Interface** | Client I/O, Keyboard Menus, Command Routing | `teloxide` (dptree, dialogue) |
| **HTTP Interop** | REST API calls to OpenCode daemon | `reqwest` (JSON) |
| **Event Streaming** | Live text deltas via HTTP long-polling | `reqwest-eventsource` |
| **Voice Processing** | Voice to text conversion | `base64`, Google Cloud Speech |
| **Markdown Parsing** | Robust CommonMark to Telegram HTML | `pulldown-cmark`, `unicodeit` |
| **Concurrency** | Async runtime, timers, polling loops | `tokio` (rt-multi-thread) |
| **Telemetry** | Application logging and debugging | `tracing`, `tracing-subscriber` |

---

## Prerequisites

Before running the interface, ensure you have the following available in your environment:

1. **Rust Toolchain:** Stable version `1.80` or higher.
2. **Telegram Bot Token:** Acquired by registering a new bot with [@BotFather](https://t.me/BotFather) on Telegram.
3. **OpenCode Daemon:** A running instance of the `opencode serve` daemon (typically bound to `http://127.0.0.1:4096`).
4. **Google Cloud API Key (Optional):** Required only if you want to use the Voice-to-Text feature. You need a key with `speech.googleapis.com` enabled.

---

## Installation

Clone the repository and compile the binary:

```bash
git clone https://github.com/hormuz-labs/puppetmaster.git
cd puppetmaster
cargo build --release
```

---

## Configuration

The bot relies on environment variables for configuration. Create a `.env` file in the root directory:

```bash
cp .env.example .env
```

Edit the `.env` file to match your environment constraints:

| Variable | Requirement | Description |
| :--- | :--- | :--- |
| `TELOXIDE_TOKEN` | **Required** | The API token provided by @BotFather. |
| `OPENCODE_SERVER_URL` | Optional | The REST endpoint of the OpenCode daemon (Defaults to `http://127.0.0.1:4096`). |
| `GOOGLE_API_KEY` | Optional | Needed for Voice-to-Text translation. |
| `GOOGLE_SPEECH_API_KEY` | Optional | Use a dedicated key strictly for Speech-to-Text, overriding the global `GOOGLE_API_KEY` if provided. |

---

## Usage

Start the bot executable:

```bash
cargo run --release
```

Once the process is running and successfully authenticates with the Telegram API, open your bot in the Telegram application and send `/start`.

### Supported Commands

- `/start` - Initializes a connection and creates a fresh OpenCode session.
- `/session` - Generates a new, blank conversational context while retaining your current project and model.
- `/project` - Surfaces an interactive menu of your 10 most recent OpenCode worktrees. Allows setting absolute paths manually.
- `/model` - Queries the OpenCode daemon for all available LLM models across active providers, presented as a clickable list.
- `/abort` - Instantly abort the AI's current text generation and tool executions.
- `/fetch <path>` - Grab a file from the host machine and send it natively over Telegram (images render visually, files as documents).
- `!<command>` - Prefix any message with an exclamation mark to execute a bash command on the host machine. e.g. `!ls -la`.
- `/help` - Displays the command reference.

Alternatively, utilize the persistent bottom keyboard menu for single-tap navigation without requiring slash commands.

---
⭐️ **Don't forget to star the repo if this was useful to you!** ⭐️
INNER_EOF
