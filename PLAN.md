# Telegram OpenCode Bot Development Plan

**Stage 1: Project Setup & Telegram Connectivity**
* Initialize the Rust project and add necessary dependencies (`teloxide`, `tokio`, `reqwest`, `reqwest-eventsource`, `dotenvy`, `tracing`).
* Set up environment variables (`TELOXIDE_TOKEN`, `OPENCODE_SERVER_URL` etc) using a `.env` file.
* Create a simple echo bot to verify that the Telegram API integration works correctly.

**Stage 2: OpenCode Integration & Simple Chat**
* Initialize the `reqwest` HTTP client.
* Create a default OpenCode session upon startup or first message using the OpenCode REST API (`POST /session`).
* Route standard Telegram text messages to the OpenCode session using `prompt_async` and SSE streaming (`reqwest-eventsource`), then relay the AI's response delta chunks back to the Telegram chat.

**Stage 3: State Management & Commands**
* Implement user-state tracking (e.g., using `teloxide` dialogues) so multiple users can interact with the bot independently.
* Introduce standard commands:
  * `/start` - Welcome message and initialization
  * `/help` - Show available commands
  * `/session` - Manage sessions (create new, list)
  * `/project` - Change the active OpenCode directory context
  * `/model` - Switch the active AI model
* Connect these commands to the respective OpenCode REST API actions.

**Stage 4: Interactive Menus (Model Switching & Config)**
* Implement Telegram inline keyboards (buttons) to create a control menu for the commands mentioned above.
* Add interactive menus for `/model`, `/session`, and `/project` for easier selection.
* Add session management buttons (e.g., aborting a run, viewing current project context).

**Stage 5: Polish, Formatting, & Edge Cases**
* Format the AI's output using Telegram's MarkdownV2/HTML parsing (especially for code blocks).
* Handle Telegram's 4096-character message limits gracefully (chunking long responses).
* Improve error handling and timeout scenarios for long-running OpenCode tasks.
