use std::fs;
use std::path::PathBuf;
use inquire::{Confirm, Text};
use std::io::Write;

pub fn get_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".puppetmaster").join(".env"))
}

pub async fn run_onboarding() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path().ok_or("Could not determine home directory")?;
    let config_dir = config_path.parent().unwrap();

    let mut existing_config = std::collections::HashMap::new();
    if config_path.exists() {
        if let Ok(iter) = dotenvy::from_path_iter(&config_path) {
            for item in iter {
                if let Ok((key, value)) = item {
                    existing_config.insert(key, value);
                }
            }
        }
        
        let use_existing = Confirm::new("Existing configuration found. Do you want to use it as-is?")
            .with_default(true)
            .prompt()?;

        if use_existing {
            println!("Using existing configuration at {:?}", config_path);
            return Ok(());
        }
    }

    println!("Setting up configuration (press Enter to keep existing values)...");
    println!("------------------------------------------");
    println!("1. TELOXIDE_TOKEN: Get this from @BotFather on Telegram.");
    println!("2. OPENCODE_SERVER_URL: The URL where your OpenCode daemon is running (default: http://127.0.0.1:4096).");
    println!("   Make sure you have run 'opencode serve' in another terminal.");
    println!("3. ALLOWED_USERS: Your Telegram ID. You can get it from @userinfobot.");
    println!("------------------------------------------\n");

    let bot_token = Text::new("Enter Telegram Bot Token (TELOXIDE_TOKEN):")
        .with_default(existing_config.get("TELOXIDE_TOKEN").map(|s| s.as_str()).unwrap_or(""))
        .with_help_message("Required: Get this from @BotFather")
        .prompt()?;

    let server_url = Text::new("Enter OpenCode Server URL (OPENCODE_SERVER_URL):")
        .with_default(existing_config.get("OPENCODE_SERVER_URL").map(|s| s.as_str()).unwrap_or("http://127.0.0.1:4096"))
        .with_help_message("The URL of your running 'opencode serve' instance")
        .prompt()?;

    let check_server = Confirm::new("Would you like to check if the OpenCode server is reachable?")
        .with_default(true)
        .prompt()?;

    if check_server {
        println!("Checking connection to {}...", server_url);
        let client = reqwest::Client::new();
        match client.get(format!("{}/global/health", server_url)).send().await {
            Ok(resp) if resp.status().is_success() => {
                println!("✅ Connected to OpenCode server successfully!");
            }
            _ => {
                println!("⚠️  Could not reach OpenCode server at {}.", server_url);
                println!("   Ensure you have started it with 'opencode serve' and the URL is correct.");
                let continue_anyway = Confirm::new("Continue with configuration anyway?")
                    .with_default(true)
                    .prompt()?;
                if !continue_anyway {
                    return Ok(());
                }
            }
        }
    }

    let default_dir = Text::new("Enter default working directory (Optional):")
        .with_default(existing_config.get("OPENCODE_DEFAULT_DIR").map(|s| s.as_str()).unwrap_or(""))
        .prompt()?;

    let allowed_users = Text::new("Enter allowed Telegram User IDs (comma-separated):")
        .with_default(existing_config.get("ALLOWED_USERS").map(|s| s.as_str()).unwrap_or(""))
        .with_help_message("Required: This secures your bot AND enables the agent to notify you. Get your ID from @userinfobot")
        .prompt()?;

    if allowed_users.is_empty() || allowed_users == "*" {
        println!("⚠️  Warning: Using '*' allows anyone to use your bot and disables agent notifications.");
        let confirm_wildcard = Confirm::new("Are you sure you want to allow everyone?")
            .with_default(false)
            .prompt()?;
        if !confirm_wildcard {
            return Box::pin(run_onboarding()).await; // Restart onboarding
        }
    }

    let unauthorized_msg = Text::new("Enter unauthorized message (Optional):")
        .with_default(existing_config.get("UNAUTHORIZED_MESSAGE").map(|s| s.as_str()).unwrap_or(""))
        .prompt()?;

    let allow_groups = Confirm::new("Allow bot in group chats?")
        .with_default(existing_config.get("ALLOW_IN_GROUPS").map(|s| s == "true").unwrap_or(true))
        .prompt()?;

    let speech_key = Text::new("Enter Google Speech API Key (Optional):")
        .with_default(existing_config.get("GOOGLE_SPEECH_API_KEY").map(|s| s.as_str()).unwrap_or(""))
        .prompt()?;

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
    }

    let mut file = fs::File::create(&config_path)?;
    writeln!(file, "TELOXIDE_TOKEN={}", bot_token)?;
    writeln!(file, "OPENCODE_SERVER_URL={}", server_url)?;
    if !default_dir.is_empty() {
        writeln!(file, "OPENCODE_DEFAULT_DIR={}", default_dir)?;
    }
    writeln!(file, "ALLOWED_USERS={}", allowed_users)?;
    if !unauthorized_msg.is_empty() {
        writeln!(file, "UNAUTHORIZED_MESSAGE={}", unauthorized_msg)?;
    }
    writeln!(file, "ALLOW_IN_GROUPS={}", allow_groups)?;
    if !speech_key.is_empty() {
        writeln!(file, "GOOGLE_SPEECH_API_KEY={}", speech_key)?;
    }

    println!("Configuration saved to {:?}", config_path);
    Ok(())
}
