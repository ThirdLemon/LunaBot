use std::collections::HashMap;
use std::sync::Arc;

use serenity::all::{ChannelId, CreateEmbed, CreateEmbedFooter, CreateMessage, GuildId, Member, PartialGuild, Role, UserId};
use serenity::model::channel::Message;

use serenity::prelude::*;

use crate::data::*;

//////////////////////////////////////////
// Functions to get data to pass around //
//////////////////////////////////////////

///Gets the data for a specific user
pub async fn get_user_data(ctx: Context, user_id: u64) -> UserData {
    return get_user_data_lock(ctx).await.read().await.data[&user_id].clone();
}

///Lets you store arbitrary data as a string on a user. Returns whether it was successful.
pub async fn set_user_storage(ctx: Context, user_id: u64, key: String, value: String) -> bool {
    if let Some(data) = get_user_data_lock(ctx).await.write().await.data.get_mut(&user_id) {
        data.user_data.insert(key, value);
        return true;
    }
    return false;
}

///Lets you grab a string from stored data on a user
pub async fn get_user_storage(ctx: Context, user_id: u64, key: String) -> Option<String> {
    return get_user_data_lock(ctx).await.read().await.data[&user_id].user_data.get(&key).cloned();
}

///Gets the data for a server, or "Guild"
pub async fn get_guild(ctx: Context, msg: Message) -> Option<PartialGuild>{
    if let Some(guild_id) = msg.guild_id {
        if let Ok(guild) = ctx.http.get_guild(guild_id).await {
            return Some(guild);
        }
    }
    return None;
}

//////////////////////
// Functions to use //
//////////////////////

///Says a message in the channel. Returns whether the message got sent.
pub async fn say(ctx: Context, channel_id: ChannelId, message: String) -> bool {
    if let Err(why) = channel_id.say(&ctx.http, message).await {
        println!("Error sending message: {why:?}");
        return false;
    }
    return true;
}

///Sends an embed in the channel. Returns whether the message got sent.
pub async fn embed(ctx: Context, channel_id: ChannelId, title: String, message: String, footer: Option<String>) -> bool{
    let mut embed = CreateEmbed::new().title(title).description(message);
    if let Some(footer_text) = footer {
        embed = embed.footer(CreateEmbedFooter::new(footer_text));
    }
    let builder = CreateMessage::new().embed(embed);
    if let Err(why) = channel_id.send_message(&ctx.http, builder).await {
        println!("Error sending message: {why:?}");
        return false;
    }
    return true;
}

///Gets the nickname of a user
pub async fn get_nickname(ctx: Context, user: UserId, guild_id: GuildId) -> Option<String>{
    if let Ok(user_data) = user.to_user(ctx.http.clone()).await {
        return user_data.nick_in(ctx.http, guild_id).await;
    }
    return None;
}

///Gets a rank's name from an amount of XP
pub fn rank(level: u64) -> String {
    let thresholds = vec![
        ("", 0),
        ("Beginner - Rank I", 1),
        ("Novice - Rank II", 2),
        ("Expert - Rank III", 3),
        ("Master - Rank IV", 4),
        ("Sage - Rank V", 5),
        ("Legend - Rank VI", 6),
        ("Myth - Rank VII", 7),
        ("Lord - Rank VIII", 8),
        ("Wizard - Rank IX", 9),
        ("Ghost - Rank X", 10),
        ("Demon - Rank XI", 11),
        ("No Life - Rank XII", 12),
        ("Grinder - Rank XIII", 13),
        ("Farmer - Rank XIV", 14),
        ("Destroyer - Rank XV", 15),
        ("Obliterator - Rank XVI", 16),
        ("Millionaire - Rank XVII", 17),
        ("Billionaire - Rank XVIII", 18),
        ("Ascendant - Rank XIX", 19),
        ("Overflowing - Rank XX", 20),
        ("Eternal - Rank XXI", 21)
    ];

    let mut ret_val = "";

    for (name, threshold) in thresholds {
        if level >= threshold {
            ret_val = name;
        }
    }

    return ret_val.to_string();
}

pub fn level_ansi_color(level: u64) -> String {
    match level {
        1 => "37",
        2 => "32",
        3 => "36",
        4 => "33",
        5 => "33",
        6 => "31",
        7 => "32",
        8 => "34",
        9 => "35",
        10 => "30",
        11 => "31",
        12 => "36",
        13 => "35",
        14 => "35",
        15 => "32",
        16 => "35",
        17 => "30",
        18 => "34",
        19 => "30",
        20 => "46;31",
        21 => "47;33",
        _ => "30"
    }.to_string()
}

///Gets a level from an amount of xp
pub fn level(xp: u64) -> u64 {
    let thresholds = vec![
        10,
        50,
        200,
        500,
        1000,
        2000,
        3250,
        5000,
        7000,
        10000,
        13500,
        17500,
        22000,
        27500,
        35000,
        45000,
        57500,
        70000,
        85000,
        105000,
        130000
    ];

    let mut ret_val = 0;

    for threshold in thresholds {
        if xp >= threshold {
            ret_val += 1;
        }
    }

    return ret_val;
}

///Returns the string needed to ping the user
pub async fn ping(user_id: UserId) -> String {
    return format!("<@{}>", u64::from(user_id));
}

///Adds a role to a member (returns if it was successful)
pub async fn add_role(ctx: Context, member: Member, guild_id: GuildId, role_name: &str) -> bool {
    if let Ok(guild) = ctx.http.get_guild(guild_id).await {
        if let Some(role) = get_role(guild, role_name).await {
            if let Ok(_) = member.add_role(ctx.clone().http, role.id).await {
                return true;
            }
        }
    }
    return false;
}

///Removes a role from a member (returns if it was successful)
pub async fn remove_role(ctx: Context, member: Member, guild_id: GuildId, role_name: &str) -> bool {
    if let Ok(guild) = ctx.http.get_guild(guild_id).await {
        if let Some(role) = get_role(guild, role_name).await {
            if let Ok(_) = member.remove_role(ctx.clone().http, role.id).await {
                return true;
            }
        }
    }
    return false;
}

//////////////////////////////////////////////////////////
// Functions that get data, but you can probably ignore //
//////////////////////////////////////////////////////////

///Gets the lock for user data, useful for writing data or reading all of it
pub async fn get_user_data_lock(ctx: Context) -> Arc<serenity::prelude::RwLock<Data>> {
    let data_lock = {
        let data_read = ctx.data.read().await;
        data_read.get::<GlobalData>().expect("Expected Data in TypeMap.").clone()
    };
    return data_lock;
}

///Gets the data for a role
pub async fn get_role(guild: PartialGuild, role_name: &str) -> Option<Role> {
    if let Some(role) = guild.role_by_name(role_name) {
        return Some(role.to_owned());
    }
    return None;
}