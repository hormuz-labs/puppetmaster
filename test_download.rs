use teloxide::{net::Download, Bot};
async fn test() {
    let bot = Bot::new("token");
    let mut buffer = Vec::new();
    bot.download_file("path", &mut buffer).await;
}
