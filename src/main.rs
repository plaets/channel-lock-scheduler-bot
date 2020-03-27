use serenity::client::Client;
use serenity::model::channel::{Message, PermissionOverwriteType, PermissionOverwrite, GuildChannel, ChannelType};
use serenity::model::guild::{Guild, Role};
use serenity::model::id::{GuildId, RoleId};
use serenity::model::permissions::Permissions;
use serenity::prelude::{EventHandler, Context};
use serenity::utils::MessageBuilder;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

#[group]
#[commands(lock, unlock, create)]
struct General;

use std::env;
use std::rc::Rc;
use std::collections::HashMap;

struct Handler;
impl EventHandler for Handler {
    fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if !is_new { return }
        println!("joined a new server");
        if let Some(ch) = guild.channels.values().filter(|c| (***c).read().kind == ChannelType::Text).nth(0) {
            if let Err(err) = (**ch).read().send_message(ctx.http.clone(), |m| m.content("hello")) { //amazing, this took so much fucking time... 
                println!("failed to send da hello message {:?}", err); 
            }
        }
    }
}

fn main() {
    println!("starting");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("No discord token specified"), Handler)
        .expect("Error creating client");
    
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .group(&GENERAL_GROUP)
        .after(|_, _, _, err| {
            if err.is_err() {
                println!("error: {:?}", err);
            }
        })
    );

    if let Err(why) = client.start() {
        println!("An error occured while running the client: {:?}", why);
    }
}

trait Task {
    fn execute(&self, ctx: &mut Context, guild: &GuildId) -> Result<(), serenity::Error>;
}

struct ReoccurringSpec {
    year: Option<u16>,
    month: Option<u16>,
    day_of_week: Option<u8>,
    day_of_month: Option<u8>,
    hours: Option<u8>,
    minutes: Option<u8>,
}

enum TaskTimeSpec {
    Once(u64),
    Reoccurring(ReoccurringSpec)
}

struct GuildConfig {
    guild: GuildId,
    management_roles: Vec<RoleId>,
    tasks: HashMap<TaskTimeSpec, Rc<dyn Task>>,
}

fn find_channel(ctx: &mut Context, guild: &GuildId, name: &str) -> Result<Option<GuildChannel>, serenity::Error> {
    Ok(guild.channels(ctx.http.clone())?.values().find(|c| c.name == name).and_then(|c| Some((*c).clone())))
    //not cloning would be epic but i dont know how to take something out of a shared reference whatever that means
}

fn find_role(ctx: &mut Context, guild: &GuildId, name: &str) -> Result<Option<Role>, serenity::Error> {
    Ok(ctx.http.get_guild_roles(*guild.as_u64())?.into_iter().find(|r| r.name == name))
}

#[command]
fn create(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild = &msg.guild_id.expect("failed to get guild_id");
    if let Ok(Some(_)) = find_channel(ctx, guild, "niedziela-wieczor") {
        msg.reply(ctx, "channel already exists")?;
    } else {
        guild.create_channel(ctx.http.clone(), |c| c.name("niedziela-wieczor"))?;
        msg.reply(ctx, "created")?;
    }

    Ok(())
}

#[command]
fn lock(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild = &msg.guild_id.expect("failed to get guild_id");
    if let Ok(Some(everyone)) = find_role(ctx, guild, "niedziela-poster") {
        if let Ok(Some(channel)) = find_channel(ctx, guild, "niedziela-wieczor") {
            channel.id.create_permission(ctx.http.clone(), &PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::SEND_MESSAGES,
                kind: PermissionOverwriteType::Role(everyone.id),
            })?;
            msg.reply(ctx, "locked the channel")?;
        } else {
            msg.reply(ctx, "channel does not exist")?;
        }
    } else {
        msg.reply(ctx, "could not find the role, wtf???")?;
    }

    Ok(())
}

#[command]
fn unlock(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild = &msg.guild_id.expect("failed to get guild_id");
    if let Ok(Some(everyone)) = find_role(ctx, guild, "niedziela-poster") {
        if let Ok(Some(channel)) = find_channel(ctx, guild, "niedziela-wieczor") {
            channel.id.create_permission(ctx.http.clone(), &PermissionOverwrite {
                allow: Permissions::SEND_MESSAGES,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(everyone.id),
            })?;
            msg.reply(ctx, "unlocked the channel")?;
        } else {
            msg.reply(ctx, "channel does not exist")?;
        }
    } else {
        msg.reply(ctx, "could not the role, wtf???")?;
    }

    Ok(())
}
