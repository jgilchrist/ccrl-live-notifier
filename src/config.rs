use crate::ccrllive::CcrlLiveRoom;

pub struct Config {
    pub webhook_url: String,
    pub rooms: Vec<CcrlLiveRoom>,
    pub engines: Vec<String>,
}