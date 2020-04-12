use serenity::model::guild::Guild;
use serenity::prelude::*;
use std::sync::{Arc};

pub struct State {
    pub guilds: Vec<(Box<Context>, Guild, u64)>, //u64 - channel id
    pub locked: bool,
    pub bot_id: u64,
}

impl State {
    pub fn new(bot_id: u64, locked: bool) -> Self {
        Self {
            guilds: Vec::new(),
            locked,
            bot_id,
        }
    }
}

pub struct StateKey;
impl TypeMapKey for StateKey {
    type Value = Arc<Mutex<State>>;
}
