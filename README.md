# CCRL Live Notifier

Get notified when games featuring your engine (or others) start on ccrl.live broadcasts.

Currently, the only supported method of notification is Discord webhook.

## Usage

Run the binary as follows:

```sh
$ cargo run -- \
    --discord-webhook [...] \
    --engines my_engine_name --engines another_engine_name \
    --rooms [ccrl room to watch]
```
