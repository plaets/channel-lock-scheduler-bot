# channel-lock-scheduler-bot

A self-hosted Discord bot for locking and unlocking a channel based on a schedule. 
This bot can join multiple Discord servers, however, it will work on the same schedule on all of them. 
**Currently, the schedule must be specified in UTC.**

## Installation

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Clone this repository
3. Run `cargo run --release` in the cloned repository
4. Create a Discord API token, add the bot to your server... (there are many guides on how to do that on the internet and I'm not sure whether i should link to any of them)
5. Modify the created configuration file
6. Run `cargo run --release` again

## Configuration file

The bot searches for `bot.cfg` in its working directory. An example configuration file will be created when the bot is run for the first time.

```toml
discord_token = 'DISCORD_TOKEN'  # discord api token
channel_name = 'example-channel' # name of the channel to lock (channel will be created if it doesn't exist)
role_name = 'example-role'       # name of the role to lock the channel for
lock_message = 'locked'          # message to post when locking the channel (the message will be posted in the channel that's being locked)
unlock_message = 'unlocked'      # message to post when unlocking the channel 
lock_on = '0 0 0 * * Mon *'      # locking schedule in crontab format ***in UTC***
unlock_on = '0 0 21 * * Sun *'   # unlocking schedule in crontab format ***in UTC***
agressive_lock = true            # if enabled, when the channel is locked, the bot will delete all messages posted to that channel 
```

## TODO

* Refactoring
* `agressive_lock` should work only if a user has the role specified in the configuration
* Logs

## License

This project is licensed under the MIT License - see the LICENSE.md file for details
