use serde_json::{json, Value};
use anyhow::Result;

pub fn send_message(webhook_url: &str, message: &str) -> Result<()> {
    call_webhook(webhook_url, json!({
        "username": "ccrl-live-notifier",
        "content": message
    }))
}

pub fn send_embed_message(webhook_url: &str, title: &str, description: &str) -> Result<()> {
    call_webhook(webhook_url, json!({
        "username": "ccrl-live-notifier",
        "allowed_mentions": { "parse": ["users"] },
        "embeds": [{
            "title": title,
            "description": description,
        }]
    }))

}

fn call_webhook(webhook_url: &str, body: Value) -> Result<()> {
    let client = reqwest::blocking::Client::new();

    client.post(webhook_url)
        .json(&body)
        .send()?
        .error_for_status()?;

    Ok(())
}