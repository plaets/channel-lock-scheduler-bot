use serenity::client::Client;
use serenity::model::channel::{ChannelType, Message};
use serenity::model::guild::{Guild, PartialGuild};
use serenity::prelude::*;
use serenity::framework::standard::StandardFramework;

use job_scheduler::*;
use confy;

use std::thread;
use std::time::Duration;
use std::sync::{Arc};
use chrono::offset; //bad idea and might just stop working one day but otherwise i cant check which job starts first 

mod config;
use config::*;

mod utils;
use utils::*;

struct State {
    guilds: Vec<(Box<Context>, Guild, u64)>, //u64 - channel id
    locked: bool,
    bot_id: u64,
}

impl State {
    fn new(bot_id: u64, locked: bool) -> Self {
        Self {
            guilds: Vec::new(),
            locked: locked,
            bot_id: bot_id,
        }
    }
}

struct StateKey;
impl TypeMapKey for StateKey {
    type Value = Arc<Mutex<State>>;
}

struct Handler;
impl EventHandler for Handler {
    fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        let state_mutex = (*ctx.data.write().get::<StateKey>().expect("Expected state")).clone();
        let mut state_guard = state_mutex.lock();

        let config = &(*(*ctx.data.read()).get::<ConfigKey>().expect("Expected config").clone());
        let channel = create_channel(&ctx, &guild, &config.channel_name).map_err(|err| println!("failed to create channel {}", err)).unwrap();
        create_role(&ctx, &guild, &config.role_name).map_err(|_| println!("failed to create role")).map_err(|err| println!("failed to create the role {:?}", err));
        (*state_guard).guilds.push((Box::new(ctx.clone()), guild.clone(), *channel.id.as_u64()));

        println!("added a new guild");
        if let Some(ch) = guild.channels.values().filter(|c| (***c).read().kind == ChannelType::Text).nth(0) {
            if !is_new { return }
            println!("joined a new server");
            if let Err(err) = (**ch).read().send_message(ctx.http.clone(), |m| m.content("hello")) { //amazing, this took so much fucking time... 
                println!("failed to send da hello message {:?}", err); 
            }
        }
    }

    fn guild_delete(&self, ctx: Context, partial_guild: PartialGuild, _: Option<Arc<serenity::prelude::RwLock<Guild>>>) {
        print!("guild delete {}", partial_guild.id.as_u64());
        let state_mutex = (*ctx.data.write().get::<StateKey>().expect("Expected state")).clone();
        let mut state_guard = state_mutex.lock();
        if let Some(id) =  (*state_guard).guilds.iter().position(|p| p.1.id == partial_guild.id) {
            (*state_guard).guilds.remove(id);
        } else {
            print!("guild {} was not registered??", partial_guild.id.as_u64());
        }
    }

    fn message(&self, ctx: Context, msg: Message) { //not the best idea performace-wise
        let config = &(*((*ctx.data.read()).get::<ConfigKey>().expect("Expected config").clone()));
        if config.agressive_lock { //i wanted to have two different handlers, one for agressive locking
            //but i dont know how to share methods between implemetations and copying guild_create
            //and guild_delete to another implemetation is just stupid
            //so yeah for every message this thing above happens, epic
            //i could also store that in the handler and access it from self
            let state_mutex = (*ctx.data.write().get::<StateKey>().expect("Expected state")).clone(); 
            let state_guard = state_mutex.lock();
            if (*state_guard).locked && 
                msg.channel_id == (*state_guard).guilds.iter().find(|p| p.1.id == msg.guild_id.unwrap()).unwrap().2 &&
                *msg.author.id.as_u64() != (*state_guard).bot_id {
                    msg.delete(ctx.http.clone()).map_err(|_| println!("failed to delete a message"));
            }
        }
    }
}

fn main() {
    let cfg: Config = confy::load_path("./bot.cfg").expect("Failed to read config");
    if cfg.discord_token == Config::default().discord_token {
        println!("Please set your discord token in the configuration file");
        return 
    }

    let unlock_spec: Schedule = cfg.unlock_on.parse().expect("Invalid unlock_on specification");
    let lock_spec: Schedule = cfg.lock_on.parse().expect("Invalid lock_on specification");
    let should_be_locked = lock_spec.upcoming(offset::Utc).next() > unlock_spec.upcoming(offset::Utc).next(); //bad assumption

    println!("starting");
    let mut client = Client::new(cfg.discord_token.clone(), Handler)
        .expect("Error creating client");

    let state = Arc::new(Mutex::new(State::new(*client.cache_and_http.http.get_current_application_info().expect("failed to get app info").id.as_u64(), true)));

    {
        let mut data = client.data.write();
        data.insert::<ConfigKey>(Arc::new(cfg.clone()));
        data.insert::<StateKey>(state.clone());
    }

    let scheduler_thread = thread::spawn(move || {
        let mut scheduler = JobScheduler::new();

        scheduler.add(Job::new(unlock_spec, || {
            println!("unlocking");
            let state = state.clone();
            let mut state_guard = state.lock();
            (*state_guard).locked = false;
            (*state_guard).guilds.iter().for_each(|p| {
                unlock_channel(&*p.0, &p.1, cfg.channel_name.as_str(), cfg.role_name.as_str()).map_err(|_| println!("failed to unlock the channel"));
                if let Ok(Some(ch)) = find_channel(&*p.0, &p.1.id, cfg.channel_name.as_str()) {
                    ch.send_message(&*p.0.http.clone(), |m| m.content(cfg.unlock_message.as_str())).expect("failed the send a message");
                }
            });
            println!("done");
        }));

        scheduler.add(Job::new(lock_spec, || {
            println!("locking");
            let state = state.clone();
            let mut state_guard = state.lock();
            (*state_guard).locked = true; 
            (*state_guard).guilds.iter().for_each(|p| {
                lock_channel(&*p.0, &p.1, cfg.channel_name.as_str(), cfg.role_name.as_str()).map_err(|_| println!("failed to lock the channel"));
                if let Ok(Some(ch)) = find_channel(&*p.0, &p.1.id, cfg.channel_name.as_str()) {
                    ch.send_message(&*p.0.http.clone(), |m| m.content(cfg.lock_message.as_str())).expect("failed to send a message");
                }
            });
            println!("done");
        }));

        loop {
            scheduler.tick();
            std::thread::sleep(Duration::from_millis(1000));
        }
    });
    
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .after(|_, _, _, err| {
            if err.is_err() {
                println!("error: {:?}", err);
            }
        })
    );

    if let Err(why) = client.start() {
        println!("An error occured while running the client: {:?}", why);
    }

    scheduler_thread.join();
}
