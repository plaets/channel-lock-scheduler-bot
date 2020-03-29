use serenity::prelude::*;
use serenity::model::{
    permissions::Permissions,
    channel::{PermissionOverwriteType, PermissionOverwrite, GuildChannel},
    guild::{Guild, Role},
    id::{GuildId},
};

pub fn find_channel(ctx: &Context, guild: &GuildId, name: &str) -> Result<Option<GuildChannel>, serenity::Error> {
    Ok(guild.channels(ctx.http.clone())?.values().find(|c| c.name == name).and_then(|c| Some((*c).clone())))
    //not cloning would be epic but i dont know how to take something out of a shared reference whatever that means
}

pub fn find_role(ctx: &Context, guild: &GuildId, name: &str) -> Result<Option<Role>, serenity::Error> {
    Ok(ctx.http.get_guild_roles(*guild.as_u64())?.into_iter().find(|r| r.name == name))
}

#[derive(Debug)]
pub enum BotError {
    SerenityError(serenity::Error),
    UnknownRole(String),
    UnknownChannel(String),
}

pub fn create_channel(ctx: &Context, guild: &Guild, channel_name: &str) -> Result<GuildChannel, serenity::Error> {
    //let channel_name = (*((*ctx.data.read()).get::<ConfigKey>().expect("Expected config").clone())).channel_name.clone(); 
    //great, im not entirely (i mean, i came up with it myself by trial and error (not proud of it)) sure what all of that means but i ~~guess~~ hope it does the job
    //i love rust

    match find_channel(ctx, &guild.id, &channel_name) {
        Ok(Some(ch)) => Ok(ch),
        Ok(None) => guild.create_channel(ctx.http.clone(), |c| c.name(channel_name)),
        Err(err) => Err(err)
    }
}

pub fn create_role(ctx: &Context, guild: &Guild, role_name: &str) -> Result<Role, serenity::Error> {
    match find_role(ctx, &guild.id, &role_name) {
        Ok(Some(r)) => Ok(r),
        Ok(None) => guild.create_role(ctx.http.clone(), |r| r.name(role_name)),
        Err(err) => Err(err),
    }
}

pub fn lock_channel(ctx: &Context, guild: &Guild, channel_name: &str, role_name: &str) -> Result<(), BotError> {
    if let Ok(Some(role)) = find_role(ctx, &guild.id, role_name) {
        if let Ok(Some(channel)) = find_channel(ctx, &guild.id, channel_name) {
            channel.id.create_permission(ctx.http.clone(), &PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::SEND_MESSAGES,
                kind: PermissionOverwriteType::Role(role.id),
            }).or_else(|err| Err(BotError::SerenityError(err)))
        } else {
            Err(BotError::UnknownChannel(String::from(channel_name)))
        }
    } else {
        Err(BotError::UnknownRole(String::from(role_name)))
    }
}

pub fn unlock_channel(ctx: &Context, guild: &Guild, channel_name: &str, role_name: &str) -> Result<(), BotError> {
    if let Ok(Some(role)) = find_role(ctx, &guild.id, role_name) {
        if let Ok(Some(channel)) = find_channel(ctx, &guild.id, channel_name) {
            channel.id.create_permission(ctx.http.clone(), &PermissionOverwrite {
                allow: Permissions::SEND_MESSAGES,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Role(role.id),
            }).or_else(|err| Err(BotError::SerenityError(err)))
        } else {
            Err(BotError::UnknownChannel(String::from(channel_name)))
        }
    } else {
        Err(BotError::UnknownRole(String::from(role_name)))
    }
}
