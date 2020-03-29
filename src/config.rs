use serde::{Serialize, Deserialize};
use std::sync::{Arc};
use serenity::prelude::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub discord_token: String,
    pub channel_name: String,
    pub role_name: String,
    pub lock_message: String,
    pub unlock_message: String,
    pub lock_on: String,
    pub unlock_on: String,
    pub agressive_lock: bool,   
}

impl ::std::default::Default for Config {
    fn default() -> Self { Self {
        discord_token: String::from("DISCORD_TOKEN"), //discord api token
        channel_name: String::from("example-channel"), //channel to lock
        role_name: String::from("example-role"), //role to lock the channel for
        lock_message: String::from("locked"), //message sent when the channel is being locked
        unlock_message: String::from("unlocked"), //message sent when the channel is being unlocked
        unlock_on: String::from("0 0 20 * * Sun *"), //unlock time specification in cron format ***in UTC***
        lock_on: String::from("0 0 0 * * Mon *"), //lock time specification in cron format ***in UTC***
        agressive_lock: true, //if enabled, the bot will delete all messages posted when the channel is locked, 
                              //useful to enforce the lock on the server admins
                              //currently this doesn't check whether the role matches or not (it
                              //probably should)
    } }
}

pub struct ConfigKey;
impl TypeMapKey for ConfigKey {
    type Value = Arc<Config>;
}
