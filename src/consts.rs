use std::path::Path;

use lazy_static::lazy_static;

pub const RUST_ZAPRET_VER: &str = "0.2.0";
pub const ZAPRET_VER: &str = "1.9.6";
pub const REPO_URL: &str = "https://github.com/maslina524/zapret-rust";
pub const ZAPRET_URL: &str = "https://github.com/Flowseal/zapret-discord-youtube";
pub const SRVCNAME: &str = "zapret-rust";
pub const GAME_FILTER_MN: u8 = 12; // only for test

lazy_static! {
    static ref CONFIGS_PATH: &'static Path = Path::new("configs/");
    static ref BIN_PATH: &'static Path = Path::new("bin/");
    static ref LISTS_PATH: &'static Path = Path::new("lists/");
}

pub fn get_configs_path() -> &'static Path {
    &CONFIGS_PATH
}

pub fn get_bin_path() -> &'static Path {
    &BIN_PATH
}

pub fn get_lists_path() -> &'static Path {
    &LISTS_PATH
}