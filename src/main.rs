use serenity::client::Client;
use serenity::prelude::*;
use serenity::framework::standard::StandardFramework;

use job_scheduler::*;
use confy;

use std::thread;
use std::time::Duration;
use chrono::offset; //bad idea and might just stop working one day but otherwise i cant check which job starts first 
use std::sync::{Arc};

mod config;
use config::*;

mod bot;
use bot::partial_permission_overwrite::*;
use bot::guild_context::*;
use bot::state::{State, StateKey};
use bot::handler::Handler;

fn main() {
    let cfg: Config = confy::load_path("./bot.cfg").expect("Failed to read config");
    if cfg.discord_token == Config::default().discord_token {
        println!("Please set your Discord API token in the configuration file");
        return 
    }

    let unlock_spec: Schedule = cfg.unlock_on.parse().expect("Invalid unlock_on specification");
    let lock_spec: Schedule = cfg.lock_on.parse().expect("Invalid lock_on specification");
    let should_be_locked = lock_spec.upcoming(offset::Utc).next() > unlock_spec.upcoming(offset::Utc).next(); //bad idea
    
    println!("next lock: {:?}", lock_spec.upcoming(offset::Utc).next());
    println!("next unlock: {:?}", unlock_spec.upcoming(offset::Utc).next());

    println!("starting");
    let mut client = Client::new(cfg.discord_token.clone(), Handler)
        .expect("Error creating client");

    let state = Arc::new(Mutex::new(State::new(*client.cache_and_http.http.get_current_application_info().expect("failed to get app info").id.as_u64(), should_be_locked)));

    {
        let mut data = client.data.write();
        data.insert::<ConfigKey>(Arc::new(cfg.clone()));
        data.insert::<StateKey>(state.clone());
    }

    let scheduler_thread = thread::spawn(move || {
        let mut scheduler = JobScheduler::new();

        let job_fn = |msg: &str, perm: PartialPermissionOverwrite, locked: bool| {
            let state = state.clone();
            let mut state_guard = state.lock();
            (*state_guard).locked = locked;
            (*state_guard).guilds.iter().for_each(|p| {
                let gctx = GuildContext{ ctx: &*p.0, guild: &p.1 };
                gctx.change_channel_permissions(cfg.channel_name.as_str(), cfg.role_name.as_str(), perm.clone()).map_err(|err| println!("failed to change perms of the channel {:?}", err)).ok();
                if let Ok(Some(ch)) = gctx.find_channel(cfg.channel_name.as_str()) {
                    ch.send_message(&*p.0.http.clone(), |m| m.content(msg)).map_err(|err| println!("failed the send a message {:?}", err)).ok();
                }
            });
        };

        scheduler.add(Job::new(unlock_spec, || {
            println!("unlocking");
            job_fn(cfg.unlock_message.as_str(), create_unlock_permisson(), false);
            println!("done");
        }));

        scheduler.add(Job::new(lock_spec, || {
            println!("locking");
            job_fn(cfg.lock_message.as_str(), create_lock_permisson(), true);
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

    scheduler_thread.join().map_err(|err| println!("failed to join the scheduler thread {:?}", err)).unwrap();
}
