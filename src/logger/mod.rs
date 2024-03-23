
// use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

use std::ops::Deref;

use crate::{cli::CliArgs, types::common::Loc};

mod types;
mod colors;
#[macro_use]
pub mod macros;
pub use types::{Level, LogEvent, LOGGER};
use types::*;


pub struct Logger{
    pub level: i8
}


impl Logger {
    pub fn new(args: &CliArgs) -> Box<Self> {
        Box::new(Self {
            level: args.verbose
        })
    }

    pub fn init(args: &CliArgs) -> anyhow::Result<()>{
        unsafe {
            types::LOGGER = Box::leak(
                Self::new(args)
            );
        }
        Ok(())
    }

    fn get_prefix(&self, level: Level) -> String {
        use colors::{BOLD, RESET, RED, YELLOW, BLUE, GREEN, MAGENTA};
        match level {
            Level::Error => format!("{BOLD}{RED}error{RESET}", ),
            Level::Warn =>  format!("{BOLD}{YELLOW}warn{RESET}", ),
            Level::Info =>  format!("{BOLD}{GREEN}info{RESET}", ),
            Level::Debug => format!("{BOLD}{BLUE}debug{RESET}", ),
            Level::Trace => format!("{BOLD}{MAGENTA}trace{RESET}", ),
        }
    }
}

impl Log for Logger {
    fn enabled(&self, level: Level) -> bool {
        match level {
            Level::Error if self.level >= 0 => true,
            Level::Warn |
            Level::Info  if self.level >= 1 => true,
            Level::Debug if self.level >= 2 => true,
            Level::Trace if self.level >= 3 => true,
            _ => false
        }
    }

    fn log(&self, event: LogEvent) {
        
        if self.enabled(event.level) {
            let modpath = if event.level > Level::Info {
                format!(" [{}]", event.module_path)
            } else {
                String::new()
            };

            if let Some(loc) = event.tags.get("loc") {
                let loc: String = (*loc.deref()).downcast_ref::<Loc>()
                                        .map_or(String::from("INVALID"), |l| l.to_string());
                println!("{} {}{modpath}: {}", loc, self.get_prefix(event.level), event.message);
            } else {
                println!("{}{modpath}: {}", self.get_prefix(event.level), event.message);
            }
        }
    }
    
    fn level(&self) -> i8 {
        self.level
    }
}