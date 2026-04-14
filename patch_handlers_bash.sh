sed -i 's/pub async fn bash_command(bot: Bot, msg: Message, dialogue: MyDialogue, cmd: String) -> HandlerResult {/pub async fn bash_command(bot: Bot, msg: Message, dialogue: MyDialogue) -> HandlerResult {/' src/handlers.rs
sed -i 's/if cmd.trim().is_empty() {/let text = msg.text().unwrap_or("");\n    let cmd = text.trim_start_matches('"'!'"').trim().to_string();\n\n    if cmd.is_empty() {/' src/handlers.rs
sed -i 's/\/bash ls -la/!ls -la/' src/handlers.rs
