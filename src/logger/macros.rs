

#[macro_export]
macro_rules! log {
    ({$($k: expr => $v: expr),* $(,)? }, $lvl:expr, $($arg:tt),+) => {
        crate::log_tagged!({$($k => $v,)*}, crate::logger::Level::Info, $($arg)+)
    };
    (module: $module:expr, $lvl:expr, $($arg:tt)+) => {
        unsafe {
            crate::logger::LOGGER.log(
                crate::logger::LogEvent {
                    level: $lvl,
                    module_path: $module.to_string(),
                    message: format!($($arg)+),
                    tags: std::collections::HashMap::new()
                }
            )
        }
    };


    ($lvl:expr, $($arg:tt)+) => {
        crate::log!(module: module_path!(), $lvl, $($arg)+)
    };
}

#[macro_export]
macro_rules! log_tagged {
    ({$($k: expr => $v: expr),* $(,)? }, $module:expr, $lvl:expr, $($arg:tt)+) => {
        unsafe {
            crate::logger::LOGGER.log(
                crate::logger::LogEvent {
                    level: $lvl,
                    module_path: $module.to_string(),
                    message: format!($($arg)+),
                    tags: map_macro::hash_map!{$(stringify!($k).to_string() => Box::new($v) as Box<dyn core::any::Any>,)*}
                }
            )
        }
    };
}


#[macro_export]
macro_rules! debug {
    (module: $module:expr, $($arg:tt)+) => {
        crate::log!(module: $module, crate::logger::Level::Debug, $($arg:tt)+)
    };
    ({$($k: expr => $v: expr),* $(,)? }, $($arg:tt)+) => {
        crate::log_tagged!({$($k => $v,)*}, module_path!(), crate::logger::Level::Debug, $($arg)+)
    };
    ({$($k: expr => $v: expr),* $(,)? }, module: $module:expr, $($arg:tt)+) => {
        crate::log_tagged!({$($k => $v,)*}, $module, crate::logger::Level::Debug, $($arg)+)
    };
    ($($arg:tt)+) => {
        crate::log!(crate::logger::Level::Debug, $($arg)+)
    };
}

#[macro_export]
macro_rules! info {
    (module: $module:expr, $($arg:tt)+) => {
        crate::log!(module: $module, crate::logger::Level::Info, $($arg:tt)+)
    };
    ({$($k: expr => $v: expr),* $(,)? }, $($arg:tt)+) => {
        crate::log_tagged!({$($k => $v,)*}, module_path!(), crate::logger::Level::Info, $($arg)+)
    };
    ({$($k: expr => $v: expr),* $(,)? }, module: $module:expr, $($arg:tt)+) => {
        crate::log_tagged!({$($k => $v,)*}, $module, crate::logger::Level::Info, $($arg)+)
    };
    ($($arg:tt)+) => {
        crate::log!(crate::logger::Level::Info, $($arg)+)
    };
}

#[macro_export]
macro_rules! warn {
    (module: $module:expr, $($arg:tt)+) => {
        crate::log!(module: $module, crate::logger::Level::Warn, $($arg:tt)+)
    };
    ({$($k: expr => $v: expr),* $(,)? }, $($arg:tt)+) => {
        crate::log_tagged!({$($k => $v,)*}, module_path!(), crate::logger::Level::Warn, $($arg)+)
    };
    ({$($k: expr => $v: expr),* $(,)? }, module: $module:expr, $($arg:tt)+) => {
        crate::log_tagged!({$($k => $v,)*}, $module, crate::logger::Level::Warn, $($arg)+)
    };
    ($($arg:tt)+) => {
        crate::log!(crate::logger::Level::Warn, $($arg)+)
    };
}

#[macro_export]
macro_rules! error {
    (module: $module:expr, $($arg:tt)+) => {
        crate::log!(module: $module, crate::logger::Level::Error, $($arg:tt)+)
    };
    ({$($k: expr => $v: expr),* $(,)? }, $($arg:tt)+) => {
        crate::log_tagged!({$($k => $v,)*}, module_path!(), crate::logger::Level::Error, $($arg)+)
    };
    ({$($k: expr => $v: expr),* $(,)? }, module: $module:expr, $($arg:tt)+) => {
        crate::log_tagged!({$($k => $v,)*}, $module, crate::logger::Level::Error, $($arg)+)
    };
    ($($arg:tt)+) => {
        crate::log!(crate::logger::Level::Error, $($arg)+)
    };
}
