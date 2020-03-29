# channel-lock-scheduler-bot

A Discord bot for locking and unlocking a channel based on a schedule

## Configuration file

The bot searches for `bot.cfg` in its working directory. It will create an example configuration file when run for the first time.

```toml
discord_token = 'DISCORD_TOKEN'  # discord api token
channel_name = 'example-channel' # name of the channel to lock (channel will be created if it doesn't exist)
role_name = 'example-role'       # name of the role to lock the channel for
lock_message = 'locked'          # message to post when locking the channel (the message will be posted in the channel that's being locked)
unlock_message = 'unlocked'      # message to post when unlocking the channel 
lock_on = '0 0 0 * * Mon *'      # locking schedule in crontab format
unlock_on = '0 0 21 * * Sun *'   # unlocking schedule in crontab format
agressive_lock = true            # if enabled, when the channel is locked, the bot will delete all messages posted to that channel 
```
