/**
 * Prints out extra information
 */
pub const DEV_MODE: bool = false;

pub const DEFAULT_OUT_FILE: &str = "a.out";
pub const DEFAULT_INCLUDES: [&str;2] = [
    "./include",
    "~/.mclang/include",
];


/**
 * Interpreting configs
 * `MEM_SZ` is the buffer size for memory
 * `STRING_SZ` is the buffer size for strings
 * if you have buffer overflow consider increasing these
 */
pub const MEM_SZ: usize = 640 * 1000; // 4kb
pub const STRING_SZ: usize = 640 * 1000; // 4kb


/**
 * Experimental options
 */
pub const ENABLE_EXPORTED_FUNCTIONS: bool = false;