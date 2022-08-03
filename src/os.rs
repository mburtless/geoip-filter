use std::time::SystemTime;

pub trait Host {
    fn debug(&self, msg: &str);
    fn info(&self, msg: &str);
    fn warn(&self, msg: &str);
    fn error(&self, msg: &str);
    fn inc(&self, m: u32);
    fn current_time(&self) -> SystemTime;
}