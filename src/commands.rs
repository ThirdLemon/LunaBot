use std::collections::HashMap;

use serenity::prelude::*;
use serenity::model::channel::Message;

use crate::commandlib::*;
use crate::data::UserData;

pub fn all_commands() -> HashMap<String, (String, Vec<String>)> {
    let mut commands: HashMap<String, (String, Vec<String>)> = HashMap::new();

    commands.insert("help".to_string(), ("Prints this page".to_string(), vec![]));
    commands.insert("leaderboard".to_string(), ("Shows XP for top users".to_string(), vec!["?Page".to_string()]));
    commands.insert("xp".to_string(), ("Says how much XP {User} has, and how much until the next level".to_string(), vec!["?User".to_string()]));
    commands.insert("xpcooldown".to_string(), ("Says how much time is left until the next XP drop for {User}".to_string(), vec!["?User".to_string()]));

    return commands;
}

pub async fn run_command(cmd: String, ctx: Context, msg: Message, args: Vec<String>) {
    match cmd.as_str() {
        "help" => help(ctx, msg, args).await,
        "leaderboard" => leaderboard(ctx, msg, args).await,
        "xp" => xp(ctx, msg, args).await,
        "xpcooldown" => xpcooldown(ctx, msg, args).await,
        _ => {
            println!("unknown command: {}", cmd);
        }
    }
}

async fn help(ctx: Context, msg: Message, args: Vec<String>) {
    let mut message = "Help:\n".to_string();
    for (cmd, (desc, args)) in all_commands() {
        let args = if args.len() > 0 {format!(" ({})", args.join(" "))} else {"".to_string()};
        message += &format!("!{}{}: {}\n", cmd, args, desc);
    }
    say(ctx, msg.channel_id, message).await;
}

async fn leaderboard(ctx: Context, msg: Message, args: Vec<String>){
    let data_lock = get_user_data_lock(ctx.clone()).await;
    let data = data_lock.read().await;

    let mut leaderboard: Vec<UserData> = data.data.values().cloned().collect();
    
    leaderboard.sort_by(|a, b| b.xp.cmp(&a.xp));

    let skip = if let Some(page) = args.first() {if let Ok(pageno) = page.parse::<usize>(){pageno.min(((leaderboard.len() as f64)/20.0) as usize + 1)}else{1}}else{1};

    embed(ctx, msg.channel_id, "XP LEADERBOARD".to_owned(), 
        format!("```ansi\n{}```", leaderboard.iter().skip((skip - 1)*20).take(20).map(|user| format!("\u{001b}[1;{}m{}: {} xp\u{001b}[0m\n", level_ansi_color(user.level), user.name.to_owned(), user.xp)).collect::<String>()),
        Some(format!("Page {} of {}", skip, ((leaderboard.len() as f64)/20.0).ceil()))).await;
}

async fn xpcooldown(ctx: Context, msg: Message, args: Vec<String>) {
    let data = get_user_data(ctx.to_owned(), msg.author.id.into()).await;
    let out = format!("{}'s XP Cooldown Expires in {} seconds", data.name, 60 - (msg.timestamp.unix_timestamp() - data.last_message_timestamp.unix_timestamp()));

    say(ctx, msg.channel_id, format!("{}", out.as_str())).await;
}

async fn xp(ctx: Context, msg: Message, args: Vec<String>) {
    

    let data = get_user_data(ctx.to_owned(), msg.author.id.into()).await;

    let mut until_next_level = 0;
    for threshold in get_level_thresholds() {
        if data.xp < threshold {
            until_next_level = threshold - data.xp;
        }
    }
    let out = format!("{} has **{}** XP. {}'s next level is in **{}** XP.", data.name, data.xp, data.name, until_next_level);

    say(ctx, msg.channel_id, format!("{}", out.as_str())).await;
}