pub async fn handle_prompt(
    bot: Bot, 
    msg: Message, 
    (session_id, _directory, model): (String, Option<String>, Option<String>), 
    client: Client, 
    server_url: Arc<String>
) -> HandlerResult {
    let chat_id = msg.chat.id;
    let mut parts = Vec::new();
    let mut text_content = String::new();
    
    if let Some(t) = msg.text() {
        text_content = t.to_string();
    } else if let Some(t) = msg.caption() {
        text_content = t.to_string();
    }

    if let Some(voice) = msg.voice() {
        let bot_msg = bot.send_message(chat_id, "🎙 Processing voice message...").await?;
        let google_api_key = env::var("GOOGLE_SPEECH_API_KEY").unwrap_or_else(|_| env::var("GOOGLE_API_KEY").unwrap_or_default());
        if google_api_key.is_empty() {
            let _ = bot.edit_message_text(chat_id, bot_msg.id, "❌ GOOGLE_SPEECH_API_KEY not set in .env. Voice messages are not supported.").await;
            return Ok(());
        }

        let file = match bot.get_file(voice.file.id.clone()).await {
            Ok(f) => f,
            Err(e) => {
                let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to get voice file: {}", e)).await;
                return Ok(());
            }
        };
        
        let temp_path = format!("/tmp/voice_{}.ogg", voice.file.id);
        let mut temp_file = match tokio::fs::File::create(&temp_path).await {
            Ok(f) => f,
            Err(e) => {
                let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to create temp file: {}", e)).await;
                return Ok(());
            }
        };

        if let Err(e) = bot.download_file(&file.path, &mut temp_file).await {
            let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to download voice: {}", e)).await;
            return Ok(());
        }

        let mut buffer = Vec::new();
        let mut temp_file = tokio::fs::File::open(&temp_path).await?;
        temp_file.read_to_end(&mut buffer).await?;
        let _ = tokio::fs::remove_file(&temp_path).await;

        let base64_audio = STANDARD.encode(&buffer);
        let payload = json!({
            "config": {
                "encoding": "OGG_OPUS",
                "sampleRateHertz": 48000,
                "languageCode": "en-US"
            },
            "audio": {
                "content": base64_audio
            }
        });

        let res = client.post(format!("https://speech.googleapis.com/v1/speech:recognize?key={}", google_api_key))
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(response) if response.status().is_success() => {
                let json_data: Value = response.json().await?;
                if let Some(results) = json_data["results"].as_array() {
                    if let Some(first) = results.first() {
                        if let Some(alternatives) = first["alternatives"].as_array() {
                            if let Some(first_alt) = alternatives.first() {
                                if let Some(transcript) = first_alt["transcript"].as_str() {
                                    text_content = transcript.to_string();
                                    let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("🎙 Transcribed:\n\n_{}_", text_content)).parse_mode(ParseMode::Markdown).await;
                                }
                            }
                        }
                    }
                }
                
                if text_content.is_empty() {
                    let _ = bot.edit_message_text(chat_id, bot_msg.id, "❌ Could not transcribe the voice message.").await;
                    return Ok(());
                }
            }
            Ok(response) => {
                let error_text = response.text().await.unwrap_or_default();
                error!("Google Speech API error: {}", error_text);
                let _ = bot.edit_message_text(chat_id, bot_msg.id, "❌ Google Speech API returned an error.").await;
                return Ok(());
            }
            Err(e) => {
                error!("Google Speech network error: {}", e);
                let _ = bot.edit_message_text(chat_id, bot_msg.id, "❌ Network error reaching Google Speech API.").await;
                return Ok(());
            }
        }
    } else if let Some(photos) = msg.photo() {
        let bot_msg = bot.send_message(chat_id, "🖼 Processing image...").await?;
        if let Some(photo) = photos.last() {
            let file = match bot.get_file(photo.file.id.clone()).await {
                Ok(f) => f,
                Err(e) => {
                    let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to get image file: {}", e)).await;
                    return Ok(());
                }
            };
            
            let mut buffer = Vec::new();
            if let Err(e) = bot.download_file(&file.path, &mut buffer).await {
                let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to download image: {}", e)).await;
                return Ok(());
            }
            
            let base64_data = STANDARD.encode(&buffer);
            let data_uri = format!("data:image/jpeg;base64,{}", base64_data);
            
            parts.push(json!({
                "type": "file",
                "mime": "image/jpeg",
                "filename": "image.jpg",
                "url": data_uri
            }));
            let _ = bot.delete_message(chat_id, bot_msg.id).await;
        }
    } else if let Some(doc) = msg.document() {
        let bot_msg = bot.send_message(chat_id, "📄 Processing document...").await?;
        let file = match bot.get_file(doc.file.id.clone()).await {
            Ok(f) => f,
            Err(e) => {
                let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to get document: {}", e)).await;
                return Ok(());
            }
        };
        
        let mut buffer = Vec::new();
        if let Err(e) = bot.download_file(&file.path, &mut buffer).await {
            let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Failed to download document: {}", e)).await;
            return Ok(());
        }
        
        let mime = doc.mime_type.as_ref().map(|m| m.to_string()).unwrap_or_else(|| "application/octet-stream".to_string());
        let filename = doc.file_name.clone().unwrap_or_else(|| "file.bin".to_string());
        
        let base64_data = STANDARD.encode(&buffer);
        let data_uri = format!("data:{};base64,{}", mime, base64_data);
        
        parts.push(json!({
            "type": "file",
            "mime": mime,
            "filename": filename,
            "url": data_uri
        }));
        let _ = bot.delete_message(chat_id, bot_msg.id).await;
    }

    if !text_content.is_empty() {
        parts.push(json!({
            "type": "text",
            "text": text_content
        }));
    }

    if parts.is_empty() {
        return Ok(());
    }

    let bot_msg = match bot.send_message(chat_id, "⏳ Thinking...").await {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to send thinking message: {}", e);
            return Ok(());
        }
    };
    
    let mut sse_req = client.get(format!("{}/event?sessionID={}", server_url, session_id));
    if let Some(ref dir) = _directory {
        sse_req = sse_req.header("x-opencode-directory", dir);
    }
    let sse_req = sse_req.try_clone().unwrap();
    
    let mut es = match EventSource::new(sse_req) {
        Ok(es) => es,
        Err(e) => {
            error!("Failed to create EventSource: {}", e);
            let _ = bot.edit_message_text(chat_id, bot_msg.id, format!("❌ Connection Error: {}", e)).await;
            return Ok(());
        }
    };

    let mut payload = json!({
