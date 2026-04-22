---
name: telegram-notify
description: Sends messages, images, videos, or documents to the user via Telegram. Useful for async communication, alerting the user when a long task is finished, or sending artifacts/screenshots directly to their phone.
---

# Telegram Notification Skill

This skill allows the agent to send updates and files directly to the user's Telegram account. It supports text messages, images, videos, audio files, and general documents.

## When to use
- The user asks you to "notify me on Telegram" when a long-running task (like a build, deployment, or script) finishes.
- You generated or downloaded an image, video, audio recording, or report, and the user wants it sent to them.
- You encounter a critical error during an asynchronous background task and need to alert the user.

## How to use
Execute the `tg-notify` binary.

### Command Signature
```bash
tg-notify "<Message or Caption>" "[Optional/Path/To/File]"
```

### Examples
Sending a simple text message:
```bash
tg-notify "✅ The database migration completed successfully."
```

Sending an image with a caption:
```bash
tg-notify "Here is the architectural diagram you requested." "./diagram.png"
```

## Configuration
The script looks for the following environment variables:
- `TELEGRAM_BOT_TOKEN`
- `TELEGRAM_CHAT_ID`

*Fallback:* If working inside the `puppetmaster` project, it will automatically attempt to use `TELOXIDE_TOKEN` and the first ID inside `ALLOWED_USERS` from the local `.env` file!
