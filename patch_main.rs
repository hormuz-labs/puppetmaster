.branch(dptree::filter(|msg: Message| msg.text() == Some("❓ Help")).endpoint(help_command))
        .branch(dptree::filter(|msg: Message| msg.text().unwrap_or("").starts_with("!")).endpoint(bash_command))
