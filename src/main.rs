use serenity::client::Client;
use serenity::model::channel::{PermissionOverwriteType, PermissionOverwrite, GuildChannel, ChannelType, Message};
use serenity::model::guild::{Guild, Role, PartialGuild};
use serenity::model::id::{GuildId};
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use serenity::framework::standard::StandardFramework;

use job_scheduler::{JobScheduler, Job, Schedule};
use serde::{Serialize, Deserialize};
use confy;

use std::thread;
use std::time::Duration;
use std::sync::{Arc};

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    discord_token: String,
    channel_name: String,
    role_name: String,
    lock_message: String,
    unlock_message: String,
    lock_on: String,
    unlock_on: String,
    agressive_lock: bool,   //delete all messages if locked
}

impl ::std::default::Default for Config {
    fn default() -> Self { Self {
        discord_token: String::from("DISCORD_TOKEN"),
        channel_name: String::from("example-channel"),
        role_name: String::from("example-role"),
        lock_message: String::from("locked"),
        unlock_message: String::from("unlocked"),
        unlock_on: String::from("0 0 20 * * Sun *"),
        lock_on: String::from("0 0 0 * * Mon *"),
        agressive_lock: true,
    } }
}

struct State {
    guilds: Vec<(Box<Context>, Guild, u64)>,
    locked: bool,
}

impl State {
    fn new() -> Self {
        Self {
            guilds: Vec::new(),
            locked: true,
        }
    }
}

struct ConfigKey;
impl TypeMapKey for ConfigKey {
    type Value = Arc<Config>;
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

        if let Some(ch) = guild.channels.values().filter(|c| (***c).read().kind == ChannelType::Text).nth(0) {
            (*state_guard).guilds.push((Box::new(ctx.clone()), guild.clone(), *(**ch).read().id.as_u64()));
            println!("added a new guild");

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
            let mut state_guard = state_mutex.lock();
            if (*state_guard).locked && msg.channel_id == (*state_guard).guilds.iter().find(|p| p.1.id == msg.guild_id.unwrap()).unwrap().2 {
                msg.delete(ctx.http.clone());
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

    let unlock_spec = cfg.unlock_on.parse().expect("Invalid unlock_on specification");
    let lock_spec = cfg.lock_on.parse().expect("Invalid lock_on specification");

    println!("starting");
    let mut client = Client::new(cfg.discord_token.clone(), Handler)
        .expect("Error creating client");

    let state = Arc::new(Mutex::new(State::new()));

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
                unlock_channel(&*p.0, &p.1, cfg.channel_name.as_str(), cfg.role_name.as_str());
                if let Ok(Some(ch)) = find_channel(&*p.0, &p.1.id, cfg.channel_name.as_str()) {
                    ch.send_message(&*p.0.http.clone(), |m| m.content(cfg.unlock_message.as_str()));
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
                lock_channel(&*p.0, &p.1, cfg.channel_name.as_str(), cfg.role_name.as_str());
                if let Ok(Some(ch)) = find_channel(&*p.0, &p.1.id, cfg.channel_name.as_str()) {
                    ch.send_message(&*p.0.http.clone(), |m| m.content(cfg.lock_message.as_str()));
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

fn find_channel(ctx: &Context, guild: &GuildId, name: &str) -> Result<Option<GuildChannel>, serenity::Error> {
    Ok(guild.channels(ctx.http.clone())?.values().find(|c| c.name == name).and_then(|c| Some((*c).clone())))
    //not cloning would be epic but i dont know how to take something out of a shared reference whatever that means
}

fn find_role(ctx: &Context, guild: &GuildId, name: &str) -> Result<Option<Role>, serenity::Error> {
    Ok(ctx.http.get_guild_roles(*guild.as_u64())?.into_iter().find(|r| r.name == name))
}

enum BotError {
    SerenityError(serenity::Error),
    UnknownRole(String),
    UnknownChannel(String),
}

fn create_channel(ctx: &mut Context, guild: &Guild, channel_name: &str) -> Result<Option<GuildChannel>, serenity::Error> {
    //let channel_name = (*((*ctx.data.read()).get::<ConfigKey>().expect("Expected config").clone())).channel_name.clone(); 
    //great, im not entirely (i mean, i came up with it myself by trial and error (not proud of it)) sure what all of that means but i ~~guess~~ hope it does the job
    //i love rust

    if let Ok(None) = find_channel(ctx, &guild.id, &channel_name) {
        guild.create_channel(ctx.http.clone(), |c| c.name(channel_name)).and_then(|r| Ok(Some(r))).or_else(|err| Err(err))
    } else {
        Ok(None)
    }
}

fn lock_channel(ctx: &Context, guild: &Guild, channel_name: &str, role_name: &str) -> Result<(), BotError> {
    //let config = (*(*ctx.data.read()).get::<ConfigKey>().expect("Expected config").clone()).clone(); 
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

fn unlock_channel(ctx: &Context, guild: &Guild, channel_name: &str, role_name: &str) -> Result<(), BotError> {
    //let config = (*((*ctx.data.read()).get::<ConfigKey>().expect("Expected config").clone())).clone(); 
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
