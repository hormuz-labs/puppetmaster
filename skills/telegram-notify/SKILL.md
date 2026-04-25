---
name: telegram-notify
description: Sends notifications, messages, images, videos, or documents to the user via Telegram. This is the PRIMARY way to send generated files (PDFs, reports, etc.) back to the user or notify them of important updates.
---

# Telegram Notification Skill

This skill allows the agent to send updates and files directly to the user's Telegram account. It supports text messages, images, videos, audio files, and general documents.

## When to use
- **Sending Files:** When you generate a file (like a PDF report, a CSV export, or a diagram) and need to "send it back" to the user.
- **Async Updates:** The user asks you to "notify me on Telegram" when a long-running task (like a build, deployment, or script) finishes.
- **Media Sharing:** You generated or downloaded an image, video, or audio recording, and the user wants it sent to them.
- **Alerts:** You encounter a critical error during an asynchronous background task and need to alert the user.

## How to use
Execute the `tg-notify` binary.

### Command Signature
```bash
tg-notify "<Message or Caption>" "[Optional/Path/To/File]"
```

### Examples
#### Sending a PDF Report
```bash
tg-notify "Here is the summary report you requested." "./output/report.pdf"
```

#### Sending a simple text message
```bash
tg-notify "✅ The database migration completed successfully."
```

#### Sending an image with a caption
```bash
tg-notify "Here is the architectural diagram you requested." "./diagram.png"
```

## Configuration
The script looks for the following environment variables:
- `TELEGRAM_BOT_TOKEN`
- `TELEGRAM_CHAT_ID`

*Fallback:* If working inside the `puppetmaster` project, it will automatically attempt to use `TELOXIDE_TOKEN` and the first ID inside `ALLOWED_USERS` from the local `.env` file!
