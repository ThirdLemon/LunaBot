use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, sync::Arc};
use serenity::all::{GuildChannel, Timestamp, UserId};

use serenity::prelude::*;

use crate::commandlib::*;

pub struct GlobalData;
impl TypeMapKey for GlobalData {
    type Value = Arc<RwLock<Data>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub data: HashMap<u64, UserData>,
    pub last_timestamp: Timestamp,
    pub changed: bool
}
impl Data {
    pub fn from_data(data: HashMap<u64, UserData>, last_timestamp: Timestamp) -> Data {
        return Data {data, last_timestamp, changed: false};
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserData {
    pub id: u64,
    pub name: String,
    pub last_message_timestamp: Timestamp,
    pub xp: u64,
    pub level: u64,
    pub user_data: HashMap<String, String>
}

pub struct DiscordHandler;
impl DiscordHandler {
    pub async fn update(&self, ctx: Context) {
        let data_lock = {
            let data_read = ctx.data.read().await;
            data_read.get::<GlobalData>().expect("Expected Data in TypeMap.").clone()
        };
        let data_guard = &data_lock.read().await;
        let data= Data::from_data(data_guard.data.clone(), data_guard.last_timestamp);
        if let Ok(filedata) = serde_json::to_string_pretty(&data) {
            if let Ok(mut file) = File::create("stats.txt"){
                if let Ok(_) = file.write_all(filedata.as_bytes()) {
                    println!("saved");
                } else {
                    println!("save failed - writing to file");
                }
            } else {
                println!("save failed - file creation");
            }
        } else {
            println!("save failed - json creation");
        }
    }
}

pub async fn update_level(ctx: Context, user_data: &mut UserData, channel: GuildChannel){
    if let Ok(guild) = ctx.http.get_guild(channel.guild_id).await {
        if let Ok(member) = ctx.http.get_member(guild.id, user_data.id.into()).await {
            if level(user_data.xp) != user_data.level {
                remove_role(ctx.clone(), member.clone(), guild.id, &rank(user_data.level)).await;
                if rank(user_data.level) < rank(level(user_data.xp)) && rank(level(user_data.xp)) != String::new() {
                    if let Ok(user) = ctx.http.get_user(UserId::from(user_data.id)).await{
                        if !user.bot {
                            if let Err(why) = channel.say(&ctx.http, format!("GG <@{}>, you just advanced to **{}** !", user_data.id, rank(level(user_data.xp)))).await {
                                println!("Error sending message: {why:?}");
                            }
                        }
                    }
                }
                user_data.level = level(user_data.xp);
            }
            if let Some(role_id) = get_role(guild.clone(), &rank(user_data.level)).await {
                if !member.roles.contains(&role_id.id)  {
                    add_role(ctx, member, guild.id, &rank(user_data.level)).await;
                }
            }
        }
    }
}