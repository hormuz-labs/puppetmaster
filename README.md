<div align="center">

<pre>
                                                    
▗▄▄ ▗  ▖▗▄▄ ▗▄▄ ▗▄▄▖▄▄▄▖    ▗  ▖ ▗▖  ▄▄ ▄▄▄▖▗▄▄▖▗▄▄ 
▐ ▝▌▐  ▌▐ ▝▌▐ ▝▌▐    ▐      ▐▌▐▌ ▐▌ ▐▘ ▘ ▐  ▐   ▐ ▝▌
▐▄▟▘▐  ▌▐▄▟▘▐▄▟▘▐▄▄▖ ▐      ▐▐▌▌ ▌▐ ▝▙▄  ▐  ▐▄▄▖▐▄▄▘
▐   ▐  ▌▐   ▐   ▐    ▐      ▐▝▘▌ ▙▟   ▝▌ ▐  ▐   ▐ ▝▖
▐   ▝▄▄▘▐   ▐   ▐▄▄▖ ▐      ▐  ▌▐  ▌▝▄▟▘ ▐  ▐▄▄▖▐  ▘
            
</pre>

**Enterprise-grade Telegram orchestration for OpenCode, engineered in Rust.**

[![Rust](https://img.shields.io/badge/rust-1.82%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![OpenCode](https://img.shields.io/badge/OpenCode-Compatible-8A2BE2?style=flat-square)](https://opencode.ai)

*Developed and maintained by **Hormuz Labs***

<br />
</div>

## Overview

**Puppetmaster** provides a secure, high-throughput asynchronous interface to your OpenCode environment. Designed for engineers and teams who require continuous access to their AI development pipelines, it allows full orchestration of tasks, session state management, and file system navigation directly from mobile or desktop Telegram clients.

Built entirely in Rust, the architecture prioritizes zero-cost abstractions, memory safety, and minimal latency—ensuring zero compromise on reliability.

---

## Core Capabilities

- **Session Orchestration:** Instantly provision, switch, and archive OpenCode sessions.
- **Contextual Awareness:** Seamlessly navigate complex file hierarchies and switch between active project roots.
- **Model Elasticity:** On-the-fly model switching with built-in memory for favorites and recent history.
- **Persistent Telemetry:** Real-time monitoring of project state, active models, and context windows via dynamic pinned messages.
- **Asynchronous Execution:** Native support for delayed prompt scheduling and recurring cron-based automations.
- **Task Interruption:** Live task abortion to preserve compute cycles and token allocation.
- **Multimodal Interfaces:** Integrated voice-to-command transcription (STT) and high-fidelity audio feedback (TTS).
- **Secure Access Control:** Strict identity verification via Telegram User IDs to prevent unauthorized execution.

---

## System Architecture

Engineered for extreme reliability, the bot leverages the modern Rust asynchronous ecosystem to handle high-concurrency workloads.

| Component | Technology | Role |
| :--- | :--- | :--- |
| **Kernel** | `tokio` | High-throughput asynchronous runtime |
| **Bot Engine** | `teloxide` | Functional reactive Telegram framework |
| **Storage** | `sqlx` + `SQLite` | Type-safe, persistent local state |
| **Integration** | `opencode-sdk-rs` | Native bindings for the OpenCode API |
| **Scheduling** | `tokio-cron` | Distributed-ready task management |

### The Rust Advantage

Benchmarked against standard Node.js implementations, this native client delivers superior operational metrics:

| Metric | Node.js (TypeScript) | Rust (Native) |
| :--- | :--- | :--- |
| **Startup Latency** | ~1200ms | **<100ms** |
| **Binary Footprint** | ~120MB+ | **~14MB** |
| **Memory Safety** | Garbage Collected | **Compile-time Guarantees** |
| **Concurrency Model** | Event Loop | **Multi-core Parallelism** |

---

## Deployment Guide

### Prerequisites
- Rust `1.82+`
- OpenCode CLI (`opencode serve`) running securely
- Telegram Bot Token & Authorized User ID

### Build & Initialize

```bash
# Clone and build the optimized release binary
git clone https://github.com/hormuzlabs/puppetmaster.git
cd puppetmaster
cargo build --release
```

### Configuration

Environment variables strictly dictate the application state. Initialize via `.env`:

```bash
cp .env.example .env
```

```env
# Core Configuration
TELEGRAM__TOKEN=your_bot_token
TELEGRAM__ALLOWED_USER_ID=your_id
OPENCODE__MODEL_ID=big-pickle

# Optional: Remote API Overrides
# OPENCODE__API_URL=http://localhost:4096
# OPENCODE__USERNAME=opencode
```

### Execution

```bash
# Execute the compiled binary
./target/release/puppetmaster
```

---

## Development & Quality Assurance

Adherence to strict coding standards is required for all upstream contributions.

```bash
# Run comprehensive test suite
cargo test

# Enforce formatting and static analysis
cargo fmt
cargo clippy -- -D warnings
```

---

<div align="center">
  <p>Licensed under the <strong>MIT License</strong>. See <a href="LICENSE">LICENSE</a> for details.</p>
  <p>2024 &copy; <strong>Hormuz Labs</strong></p>
</div>