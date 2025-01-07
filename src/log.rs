use crate::discord;

pub trait Logger {
    fn info(&self, msg: &str);
    fn error(&self, msg: &str);
}

pub struct StdoutLogger;

impl Logger for StdoutLogger {
    fn info(&self, msg: &str) {
        println!("{}", msg);
    }

    fn error(&self, msg: &str) {
        eprintln!("{}", msg);
    }
}

pub struct DiscordLogger {
    log_webhook: String,
}

impl DiscordLogger {
    pub fn new(log_webhook: String) -> DiscordLogger {
        Self { log_webhook }
    }
}

impl Logger for DiscordLogger {
    fn info(&self, msg: &str) {
        println!("{}", msg);

        let _ = discord::send_message(&self.log_webhook, msg);
    }

    fn error(&self, msg: &str) {
        eprintln!("{}", msg);

        let _ = discord::send_message(&self.log_webhook, &(":red_circle:".to_string() + msg));
    }
}
