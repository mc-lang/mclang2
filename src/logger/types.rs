use std::{any::Any, collections::HashMap, fmt::Debug };


pub static mut LOGGER: &dyn Log = &NopLogger;

struct NopLogger;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Level {
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

// pub trait Tag: Display + Debug + Any {}


pub struct LogEvent {
    pub level: Level,
    pub module_path: String,
    pub message: String,
    pub tags: HashMap<String, Box<dyn Any>>
}


impl Log for NopLogger {
    fn enabled(&self, _: Level) -> bool {false}
    fn level(&self) -> i8 {0}
    fn log(&self, _: LogEvent) {}
}


pub trait Log {
    fn enabled(&self, level: Level) -> bool;
    fn log(&self, event: LogEvent);
    fn level(&self) -> i8;
}