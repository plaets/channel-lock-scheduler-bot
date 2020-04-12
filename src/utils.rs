use serenity::prelude::*;
use serenity::model::{
    permissions::Permissions,
    channel::{PermissionOverwriteType, GuildChannel, PermissionOverwrite},
    guild::{Guild, Role},
};

#[derive(Copy, Clone)]
pub struct PartialPermissionOverwrite {
    allow: Permissions,
    deny: Permissions,
}

pub fn create_lock_permisson() -> PartialPermissionOverwrite {
    PartialPermissionOverwrite {
        allow: Permissions::empty(),
        deny: Permissions::SEND_MESSAGES,
    }
}

pub fn create_unlock_permisson() -> PartialPermissionOverwrite {
    PartialPermissionOverwrite {
        allow: Permissions::SEND_MESSAGES,
        deny: Permissions::empty(),
    }
}

impl PartialPermissionOverwrite {
    pub fn to_permission_overwrite(&self, kind: PermissionOverwriteType) -> PermissionOverwrite {
        PermissionOverwrite {
            allow: self.allow,
            deny: self.deny,
            kind,
        }
    }
}

#[derive(Debug)]
pub enum GuildContextError {
    SerenityError(serenity::Error),
    UnknownRole(String),
    UnknownChannel(String),
}

pub struct GuildContext<'a> {
    pub ctx: &'a Context,
    pub guild: &'a Guild,
}

impl<'a> GuildContext<'a> {
    pub fn change_channel_permissions(&self, channel_name: &str, role_name: &str, perm: PartialPermissionOverwrite) -> Result<(), GuildContextError> {
        if let Ok(Some(role)) = self.find_role(role_name) {
            if let Ok(Some(channel)) = self.find_channel(channel_name) {
                channel.id.create_permission(self.ctx.http.clone(), 
                 &perm.to_permission_overwrite(PermissionOverwriteType::Role(role.id)))
                    .or_else(|err| Err(GuildContextError::SerenityError(err)))
            } else {
                Err(GuildContextError::UnknownChannel(String::from(channel_name)))
            }
        } else {
            Err(GuildContextError::UnknownRole(String::from(role_name)))
        }
    }

    pub fn create_channel(&self, channel_name: &str) -> Result<GuildChannel, serenity::Error> {
        match self.find_channel(&channel_name) {
            Ok(Some(ch)) => Ok(ch),
            Ok(None) => self.guild.create_channel(self.ctx.http.clone(), |c| c.name(channel_name)),
            Err(err) => Err(err),
        }
    }

    pub fn create_role(&self, role_name: &str) -> Result<Role, serenity::Error> {
        match self.find_role(&role_name) {
            Ok(Some(r)) => Ok(r),
            Ok(None) => self.guild.create_role(self.ctx.http.clone(), |r| r.name(role_name)),
            Err(err) => Err(err),
        }
    }

    pub fn find_channel(&self, name: &str) -> Result<Option<GuildChannel>, serenity::Error> {
        Ok(self.guild.channels(self.ctx.http.clone())?.values().find(|c| c.name == name).map(|c| (*c).clone()))
    }

    pub fn find_role(&self, name: &str) -> Result<Option<Role>, serenity::Error> {
        Ok(self.ctx.http.get_guild_roles(*(self.guild.id.as_u64()))?.into_iter().find(|r| r.name == name))
    }
}
