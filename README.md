# CCRL Live Notifier

Get notified when games featuring your engine (or others) start on ccrl.live broadcasts.

Currently, the only supported method of notification is Discord webhook.

## Usage

### Finding your User ID

Discord webhooks can only mention you via your user ID. To find your Discord user ID:

- Go to Discord settings > Advanced
- Toggle 'Developer Mode'
- Right click on your name in the Users pane
- Click 'Copy User ID'

### Configuration

Add your user ID to the config file, with any engine names you're interested in. For example:

```json
{
  "users": {
    "myuserid": [
      "my_engine_name"
    ]
  }
}
```
