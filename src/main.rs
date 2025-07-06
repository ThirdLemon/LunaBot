use std::{env, fs::File, io::Read, sync::Arc};
use dotenv::dotenv;

use serenity::prelude::*;

mod data;
mod handler;
mod commands;
mod commandlib;

use crate::data::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Try to load data from backup file
    let data;
    if let Ok(mut file) = File::open("stats.txt") {
        let mut contents = String::new();
        if let Ok(_) = file.read_to_string(&mut contents) {
            if let Ok(d) = serde_json::from_str(&contents) {
                data = d;
            }else {
                println!("error parsing json");
                return;
            }
        } else {
            println!("error reading file");
            return;
        }
    } else {
        println!("error opening file");
        return;
    }

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(DiscordHandler).await.expect("Err creating client");

    {
        let mut clientdata = client.data.write().await;
        clientdata.insert::<GlobalData>(Arc::new(RwLock::new(data)));
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}