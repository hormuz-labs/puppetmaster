# OpenCode Telegram Bot (Rust)

A Telegram bot client for OpenCode that allows you to run and monitor AI coding tasks from your phone. Built with Rust for performance and reliability.

## Features

- 🤖 **Telegram Bot**: Full-featured bot with commands and inline keyboards
- 💬 **Session Management**: Create, switch, and manage OpenCode sessions
- 📁 **Project Management**: Switch between projects easily
- 🎯 **Model Selection**: Choose models from favorites and recent history
- ⏰ **Scheduled Tasks**: Schedule prompts to run later or on recurring intervals
- 🎤 **Voice Support**: Transcribe voice messages (requires STT configuration)
- 🔊 **Text-to-Speech**: Enable audio replies (requires TTS configuration)
- 🌍 **Multi-language**: Support for 6 locales (en, de, es, fr, ru, zh)
- 📊 **Live Status**: Pinned messages with current project, model, and context usage
- 🔒 **Security**: Strict user ID whitelist

## Prerequisites

- Rust 1.82+ (install from [rustup.rs](https://rustup.rs/))
- OpenCode CLI running locally (`opencode serve`)
- Telegram Bot Token (from @BotFather)
- Your Telegram User ID (from @userinfobot)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/opencode-telegram-bot-rs.git
cd opencode-telegram-bot-rs

# Build in release mode
cargo build --release

# The binary will be at target/release/opencode-telegram
```

### Quick Start

1. Copy the example environment file:
```bash
cp .env.example .env
```

2. Edit `.env` with your configuration:
```bash
# Required settings
TELEGRAM__TOKEN=your_bot_token_here
TELEGRAM__ALLOWED_USER_ID=your_user_id_here
OPENCODE__MODEL_PROVIDER=opencode
OPENCODE__MODEL_ID=big-pickle
```

3. Start OpenCode server:
```bash
opencode serve
```

4. Run the bot:
```bash
cargo run --release
# Or use the binary directly:
# ./target/release/opencode-telegram
```

## Configuration

All configuration is done via environment variables (or `.env` file):

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `TELEGRAM__TOKEN` | Bot token from @BotFather | Yes | - |
| `TELEGRAM__ALLOWED_USER_ID` | Your Telegram user ID | Yes | - |
| `TELEGRAM__PROXY_URL` | Proxy for Telegram API | No | - |
| `OPENCODE__API_URL` | OpenCode server URL | No | `http://localhost:4096` |
| `OPENCODE__USERNAME` | OpenCode auth username | No | `opencode` |
| `OPENCODE__PASSWORD` | OpenCode auth password | No | - |
| `OPENCODE__MODEL_PROVIDER` | Default model provider | Yes | - |
| `OPENCODE__MODEL_ID` | Default model ID | Yes | - |
| `BOT__LOCALE` | Bot UI language | No | `en` |
| `BOT__TASK_LIMIT` | Max scheduled tasks | No | `10` |
| `STT__API_URL` | Speech-to-text API URL | No | - |
| `STT__API_KEY` | STT API key | No | - |
| `TTS__API_URL` | Text-to-speech API URL | No | - |
| `TTS__API_KEY` | TTS API key | No | - |

## Commands

| Command | Description |
|---------|-------------|
| `/start` | Start the bot and show welcome message |
| `/help` | Show available commands |
| `/status` | Show server health, current project, session, and model |
| `/new` | Create a new session |
| `/sessions` | Browse and switch between recent sessions |
| `/projects` | Switch between OpenCode projects |
| `/commands` | Browse and run custom commands |
| `/abort` | Abort the current task |
| `/task` | Create a scheduled task |
| `/tasklist` | Browse and delete scheduled tasks |
| `/tts` | Toggle audio replies |
| `/rename` | Rename the current session |
| `/opencode_start` | Start the OpenCode server (placeholder) |
| `/opencode_stop` | Stop the OpenCode server (placeholder) |

## Project Structure

```
src/
├── main.rs              # Entry point
├── config.rs            # Configuration management
├── error.rs             # Error types
├── i18n/                # Internationalization
│   ├── mod.rs
│   └── keys.rs
├── bot/                 # Telegram bot
│   ├── mod.rs
│   ├── commands.rs      # Command handlers
│   ├── handlers.rs      # Message handlers
│   └── middleware.rs    # Auth middleware
├── opencode/            # OpenCode SDK wrapper
│   ├── mod.rs
│   ├── client.rs
│   └── types.rs
├── settings/            # Persistent settings
│   ├── mod.rs
│   └── db.rs
├── scheduled_task/      # Cron task system
│   ├── mod.rs
│   └── db.rs
└── utils/               # Utilities

locales/                 # Translation files
├── en.toml
├── ru.toml
└── ...

migrations/              # Database migrations
└── 001_initial.sql
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Check
cargo check
```

## Architecture

This bot is built with:

- **teloxide**: Elegant Telegram bots framework for Rust
- **opencode-sdk-rs**: Official OpenCode SDK for Rust
- **sqlx**: Async SQL toolkit with SQLite
- **tokio**: Async runtime
- **tokio-cron-scheduler**: Cron job scheduling
- **rust-i18n**: Internationalization

## Differences from TypeScript Version

| Aspect | TypeScript | Rust |
|--------|-----------|------|
| Runtime | Node.js | Native binary |
| Memory Safety | GC | Compile-time guarantees |
| Concurrency | Event loop | True parallelism with tokio |
| Error Handling | try/catch | Result<T, E> |
| Type Safety | Runtime | Compile-time |
| Binary Size | ~100MB+ | ~10-20MB |
| Startup Time | ~1s | ~100ms |

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Troubleshooting

### Bot doesn't respond to messages
- Check that `TELEGRAM__ALLOWED_USER_ID` matches your actual Telegram user ID
- Verify the bot token is correct
- Check logs with `RUST_LOG=debug cargo run`

### "OpenCode server is not available"
- Ensure `opencode serve` is running
- Check that `OPENCODE__API_URL` points to the correct address
- Verify authentication credentials if set

### Database errors
- The bot automatically creates its data directory
- Check permissions for the data directory
- On Linux/macOS: `~/.local/share/opencode-telegram-bot/`
- On Windows: `%APPDATA%\opencode-telegram-bot\`

## Support

- OpenCode: https://opencode.ai
- Telegram Bot API: https://core.telegram.org/bots/api
- Teloxide: https://docs.rs/teloxide
