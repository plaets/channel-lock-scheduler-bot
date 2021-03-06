# channel-lock-scheduler-bot

A self-hosted Discord bot for locking and unlocking a channel based on a schedule. 
This bot can join multiple Discord servers, however, it will work on the same schedule on all of them. 
**Currently, the schedule must be specified in UTC.**

## Installation

Binaries for x64 Linux and Windows are available on the [releases page](https://github.com/plaets/channel-lock-scheduler-bot/releases)

### Compiling from the source

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Clone this repository
3. Run `cargo run --release` in the cloned repository
4. Create a Discord API token, add the bot to your server... (there are many guides on how to do that on the internet and I'm not sure whether I should link any of them)
5. Modify the created configuration file (a file named `bot.cfg` should appear in your working directory after running `cargo run --release`)
6. Run `cargo run --release` again

### Cross-compilation

For cross-compilation I recommend [cross](https://github.com/rust-embedded/cross)

1. [Install cross](https://github.com/rust-embedded/cross#installation)
2. Run `cross build --target arm-unknown-linux-musleabihf --release`, replace `arm-unknown-linux-musleabihf` with the target (`arm-unknown-linux-musleabihf` is ok for Raspberry Pi 3)

## Running the bot

1. [Download](https://github.com/plaets/channel-lock-scheduler-bot/releases) or compile latest version of the bot
2. Generate a Discord API token, example tutorial: https://www.writebots.com/discord-bot-token/
3. Create a `bot.cfg` file, replace `DISCORD_TOKEN` with the token you got from the tutorial
4. Start the bot
5. Generate and use the bot invitation link (step 5 of the tutorial from step 2). This bot will need permissions for sending messages, managing roles and channels, viewing channels, sending, managing and reading messages. 

## Configuration file

The bot searches for `bot.cfg` in its working directory. An example configuration file will be created when the bot is run for the first time.

```toml
discord_token = 'DISCORD_TOKEN'  # discord api token
channel_name = 'example-channel' # name of the channel to lock (channel will be created if it doesn't exist)
role_name = 'example-role'       # name of the role that should be locked out of the channel
lock_message = 'locked'          # message to post when locking the channel (the message will be posted in the channel that's being locked)
unlock_message = 'unlocked'      # message to post when unlocking the channel 
lock_on = '0 0 0 * * Mon *'      # locking schedule in crontab format, ***in UTC***
unlock_on = '0 0 21 * * Sun *'   # unlocking schedule in crontab format, ***in UTC***
agressive_lock = true            # if enabled, when the channel is locked, the bot will delete all messages posted to that channel 
```

## Dependencies

* [serenity](https://github.com/serenity-rs/serenity) - library for the Discord API
* [job_scheduler](https://github.com/lholden/job_scheduler/)- for scheduling tasks
* [confy](https://github.com/rust-cli/confy) - saving and loading configuration files with serde
* [serde](https://github.com/serde-rs/serde) - used by confy to serialize and deserialize the configuration file
* [chrono](https://github.com/chronotope/chrono) - I believe it's needed to use the `upcoming` method of `Schedule` from job_scheduler

## TODO

* Refactoring
* Managing multiple channels
* Server emojis 
* `agressive_lock` should work only if a user has the role specified in the configuration
* Logs
* Daylight saving time 

## License

This project is licensed under the MIT License - see the LICENSE.md file for details

