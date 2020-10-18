use serenity::prelude::*;
use serenity::model::channel::{ChannelType, Message};
use serenity::model::guild::{Guild, PartialGuild};

use std::sync::{Arc};

use crate::config::*;
use crate::bot::partial_permission_overwrite::*;
use crate::bot::guild_context::*;
use crate::bot::state::StateKey;

pub struct Handler;
impl EventHandler for Handler {
    fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        let gctx = GuildContext{ ctx: &ctx, guild: &guild };
        let state_mutex = (*ctx.data.write().get::<StateKey>().expect("Expected state")).clone();
        let mut state_guard = state_mutex.lock();

        if (*state_guard).guilds.iter().any(|g| g.1.id == guild.id) {
            println!("guild_create event for an already existing guild {:?}", guild.id);
            return;
        }

        let config = &(*(*ctx.data.read()).get::<ConfigKey>().expect("Expected config").clone());
        match gctx.create_channel(&config.channel_name) {
            Ok(channel) => {
                if let Err(err) = gctx.create_role(&config.role_name) {
                    println!("failed to create role {}", err);
                    return
                }

                (*state_guard).guilds.push((Box::new(ctx.clone()), guild.clone(), *channel.id.as_u64()));

                println!("added a new guild {:?} {:?}", guild.name, guild.id);
                if let Some(ch) = guild.channels.values().filter(|c| (***c).read().kind == ChannelType::Text).nth(0) {
                    if !is_new { return }
                    println!("joined a new server {:?} {:?}", guild.name, guild.id);
                    (**ch).read().send_message(ctx.http.clone(), |m| m.content("hello")).map_err(|err| println!("failed to send the hello message {:?}", err)).ok();

                    if (*state_guard).locked {
                        gctx.change_channel_permissions(&config.channel_name, &config.role_name, create_lock_permisson()).map_err(|err| println!("failed to lock a channel {:?}", err)).ok();
                    } else {
                        gctx.change_channel_permissions(&config.channel_name, &config.role_name, create_unlock_permisson()).map_err(|err| println!("failed to unlock a channel {:?}", err)).ok();
                    }
                }
            }
            Err(err) => println!("failed to create channel {}", err)
        }
    }

    fn guild_delete(&self, ctx: Context, partial_guild: PartialGuild, _: Option<Arc<serenity::prelude::RwLock<Guild>>>) {
        print!("deleted from guild {:?} {:?}", partial_guild.name, partial_guild.id.as_u64());
        let state_mutex = (*ctx.data.write().get::<StateKey>().expect("Expected state")).clone();
        let mut state_guard = state_mutex.lock();
        if let Some(id) =  (*state_guard).guilds.iter().position(|p| p.1.id == partial_guild.id) {
            (*state_guard).guilds.remove(id);
        } else {
            print!("guild {:?} {:?} was not registered??", partial_guild.id.as_u64(), partial_guild.name);
        }
    }

    fn message(&self, ctx: Context, msg: Message) { //not the best idea performace-wise
        if (*((*ctx.data.read()).get::<ConfigKey>().expect("Expected config").clone())).agressive_lock { 
            let state_mutex = (*ctx.data.write().get::<StateKey>().expect("Expected state")).clone(); 
            let state_guard = state_mutex.lock();
            if (*state_guard).locked && 
                msg.channel_id == (*state_guard).guilds.iter().find(|p| p.1.id == msg.guild_id.unwrap()).unwrap().2 &&
                *msg.author.id.as_u64() != (*state_guard).bot_id {
                    msg.delete(ctx.http).map_err(|_| println!("failed to delete a message")).ok();
            }
        }
    }
}
