use teloxide::{net::Download, Bot};
#[tokio::main]
async fn main() {
    let bot = Bot::new("token");
    let mut buffer = Vec::new();
    let _ = bot.download_file("path", &mut buffer).await;
}
