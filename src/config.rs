/**
 * Prints out extra information
 */
pub const DEV_MODE: bool = true;

pub const DEFAULT_OUT_FILE: &str = "a.out";
pub const DEFAULT_INCLUDES: [&str;2] = [
    "./include",
    "~/.mclang/include",
];


/**
 * Experimental options
 */
pub const ENABLE_EXPORTED_FUNCTIONS: bool = false;