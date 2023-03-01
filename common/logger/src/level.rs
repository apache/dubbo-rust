use crate::Level;

pub(crate) struct LevelWrapper {
    pub(crate) inner: Level,
}
impl LevelWrapper {
    pub fn new(level: Level) -> Self {
        LevelWrapper { inner: level }
    }
}

impl From<String> for LevelWrapper {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str().trim() {
            "error" => LevelWrapper::new(Level::ERROR),
            "warn" => LevelWrapper::new(Level::WARN),
            "info" => LevelWrapper::new(Level::INFO),
            "debug" => LevelWrapper::new(Level::DEBUG),
            "trace" => LevelWrapper::new(Level::TRACE),
            _ => LevelWrapper::new(Level::INFO),
        }
    }
}
