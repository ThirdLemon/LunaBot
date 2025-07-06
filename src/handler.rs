use serenity::all::{Channel, Timestamp, UserId};
use serenity::futures::StreamExt;
use serenity::{async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::builder::{CreateEmbed, CreateMessage};
use std::collections::HashMap;
use regex::Regex;

use serenity::prelude::*;

use crate::data::*;
use crate::commandlib::*;
use crate::commands::all_commands;
use crate::commands::run_command;

#[async_trait]
impl EventHandler for DiscordHandler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        {
            let user_id = u64::from(msg.author.id);

            let data_lock = {
                let data_read = ctx.data.read().await;
                data_read.get::<GlobalData>().expect("Expected Data in TypeMap.").clone()
            };
            {
                let mut data = data_lock.write().await;
                if let Some(user) = data.data.get_mut(&user_id) {
                    user.name = msg.author.display_name().to_string();
                    if msg.timestamp.unix_timestamp() - user.last_message_timestamp.unix_timestamp() > 60 {
                        user.last_message_timestamp = msg.timestamp;
                        user.xp += 1;
                        data.changed = true;
                    }
                    if data.last_timestamp.unix_timestamp() < msg.timestamp.unix_timestamp() {
                        data.last_timestamp = msg.timestamp;
                    }
                } else {
                    data.data.insert(user_id, UserData { id: user_id, name: msg.author.display_name().to_string(), last_message_timestamp: msg.timestamp, xp: 1, level: level(1), user_data: HashMap::new() });
                    data.changed = true;
                }
                if let Some(user) = data.data.get_mut(&user_id) {
                    if let Ok(channel) = msg.channel(&ctx.http).await {
                        if let Some(guildchannel) = channel.guild() {
                            update_level(ctx.clone(), user, guildchannel).await;
                        }
                    }
                }
            }
        }
        if let Ok(current_user) = ctx.http.get_current_user().await {
            if current_user.id != msg.author.id {
                for (command_name, command_data) in all_commands() {
                    let re = Regex::new(r"^!(\w+)(?: (\w+))*").unwrap();
                    if let Some(captures) = re.captures(&msg.content.to_owned()) {
                        if captures[1].to_string() == command_name {
                            run_command(command_name, ctx.clone(), msg.clone(), captures.iter().skip(2).map(|c| if c.is_some() {c.unwrap().as_str().to_owned()} else {"".to_string()}).collect()).await;
                            break;
                        }
                    }
                }
            }
        }
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an authentication error, or lack
            // of permissions to post in the channel, so log to stdout when some error happens,
            // with a description of it.
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
        {
            let changed;
            {
                let data_lock = {
                    let data_read = ctx.data.read().await;
                    data_read.get::<GlobalData>().expect("Expected Data in TypeMap.").clone()
                };
                let data = data_lock.read().await;
                changed = data.changed;
            }
            if changed {
                self.update(ctx).await;
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        {
            let data_lock = {
                let data_read = ctx.data.read().await;
                data_read.get::<GlobalData>().expect("Expected Data in TypeMap.").clone()
            };
            let mut data = data_lock.write().await;

            let mut message_vec: Vec<(u64, Timestamp, String, Option<Channel>)> = vec![];

            println!("step 1, checking for new messages");
            if let Ok(guilds) = ctx.http.clone().get_guilds(None, None).await {
                for guild_info in guilds {
                    println!("checking guild {:?}", guild_info.name);
                    if let Ok(guild) = ctx.http.clone().get_guild(guild_info.id).await {
                        if let Ok(channels) = guild.channels(ctx.http.clone()).await {
                            for channel in channels.keys() {
                                println!("checking channel {:?}", channel.name(&ctx.http).await);
                                let mut messages = channel.messages_iter(ctx.http.clone()).boxed();
                                while let Some(message_result) = messages.next().await {
                                    println!("checking message");
                                    if message_vec.len() > 10000 {
                                        break;
                                    }
                                    match message_result {
                                        Ok(message) => {
                                            if message.timestamp > data.last_timestamp {
                                                if let Ok(channel) = message.channel(ctx.http.clone()).await {
                                                    message_vec.push((u64::from(message.author.id), message.timestamp, message.author.display_name().to_string(), Some(channel)));
                                                } else {
                                                    message_vec.push((u64::from(message.author.id), message.timestamp, message.author.display_name().to_string(), None));
                                                }
                                            }else{
                                                break;
                                            }
                                        },
                                        Err(error) => {
                                            eprintln!("Uh oh! Error: {}", error);
                                            break;
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }
            println!("step 2, {} messages to sift through", message_vec.len());
            message_vec.sort_by_key(|k| k.1);
            for (id, timestamp, name, channel) in message_vec {
                if let Some(user) = data.data.get_mut(&id) {
                    if user.last_message_timestamp.unix_timestamp() + 60 < timestamp.unix_timestamp() {
                        user.last_message_timestamp = timestamp;
                        user.xp += 1;
                        if level(user.xp) != user.level && channel.is_some() {
                            if let Some(guildchannel) = channel.unwrap().guild() {
                                update_level(ctx.clone(), user, guildchannel).await;
                            }
                        }
                    }
                } else {
                    data.data.insert(id, UserData { id: id, name: name, last_message_timestamp: timestamp, xp: 1, level: level(1), user_data: HashMap::new() });
                    if let Some(user) = data.data.get_mut(&id) {
                        if let Some(guildchannel) = channel.unwrap().guild() {
                            update_level(ctx.clone(), user, guildchannel).await;
                        }
                    }
                }
                if data.last_timestamp.unix_timestamp() < timestamp.unix_timestamp() {
                    data.last_timestamp = timestamp;
                }
            }
            //scrape usernames on boot
            /*
            for (userid, _) in &data.data.to_owned() {
                if let Some(userdata) = data.data.get_mut(&userid) {
                    if let Ok(user) = ctx.http.get_user(UserId::from(userid.to_owned())).await {
                        userdata.name = user.display_name().to_owned();
                    }
                }
            }
            */
        }


        self.update(ctx).await;
    }
}