use crate::config::Config;
use crate::{discord, log};

pub fn get_logger(config: &Config) -> Box<dyn Logger> {
    match config.log_webhook {
        None => Box::new(log::StdoutLogger),
        Some(ref hook) => Box::new(log::DiscordLogger::new(hook.clone())),
    }
}

pub trait Logger {
    fn info(&self, msg: &str);
    fn error(&self, msg: &str);
}

impl Logger for Box<dyn Logger + '_> {
    fn info(&self, msg: &str) {
        (**self).info(msg)
    }

    fn error(&self, msg: &str) {
        (**self).error(msg)
    }
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

        let _ = discord::send_message(
            &self.log_webhook,
            &("<@!106120945231466496> :red_circle:".to_string() + msg),
        );
    }
}
