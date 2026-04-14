# OpenCode Telegram Interface

[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-blue.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Teloxide Framework](https://img.shields.io/badge/framework-teloxide--0.17-black.svg?style=flat-square)](https://github.com/teloxide/teloxide)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)
[![Status](https://img.shields.io/badge/status-active-success.svg?style=flat-square)](#)

> A robust, high-performance Telegram bot acting as a remote control and conversational interface for the [OpenCode AI platform](https://github.com/alpha-innovation-labs/opencode-rs-sdk).

This repository provides a seamless integration between Telegram and a locally running OpenCode daemon. It leverages Rust's asynchronous ecosystem to deliver live AI streaming responses, sophisticated state management for multiple concurrent users, and interactive UI elements natively within the Telegram client.

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

### Real-Time AI Streaming
Uses Server-Sent Events (SSE) to capture output deltas from the OpenCode LLM in real-time. Responses are batched and flushed to the Telegram API sequentially to produce a smooth, typing-like experience without tripping rate limits.

### Intelligent Payload Handling
Bypasses Telegram's strict 4096-character message limits natively. Long responses (such as complete code file generation) are dynamically split into discrete, sequential message bubbles on-the-fly.

### Safe Markdown/HTML Parsing
Employs a custom Markdown-to-HTML interpreter designed specifically for code blocks. It ensures that standard Markdown syntax (like triple-backticks) is properly translated into Telegram-compatible `<pre><code>` HTML blocks, failing gracefully back to raw text if syntax errors occur.

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
| **Concurrency** | Async runtime, timers, polling loops | `tokio` (rt-multi-thread) |
| **Telemetry** | Application logging and debugging | `tracing`, `tracing-subscriber` |

---

## Prerequisites

Before running the interface, ensure you have the following available in your environment:

1. **Rust Toolchain:** Stable version `1.80` or higher.
2. **Telegram Bot Token:** Acquired by registering a new bot with [@BotFather](https://t.me/BotFather) on Telegram.
3. **OpenCode Daemon:** A running instance of the `opencode serve` daemon (typically bound to `http://127.0.0.1:4096`).

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

The bot relies on environment variables for configuration. Copy the provided `.env.example` to `.env` in the root directory:

```bash
cp .env.example .env
```

Edit the `.env` file to match your environment constraints:

| Variable | Requirement | Description |
| :--- | :--- | :--- |
| `TELOXIDE_TOKEN` | **Required** | The API token provided by @BotFather. |
| `OPENCODE_SERVER_URL` | Optional | The REST endpoint of the OpenCode daemon (Defaults to `http://127.0.0.1:4096`). |

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
- `/help` - Displays the command reference.

Alternatively, utilize the persistent bottom keyboard menu for single-tap navigation without requiring slash commands.