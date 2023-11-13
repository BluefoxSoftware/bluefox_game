use std::path::PathBuf;

use internal_bluefox_lib::plugin::load_plugins;
use lazy_static::lazy_static;

lazy_static! {
    static ref PLUGINS_DIR: PathBuf = dirs::data_local_dir().unwrap().join("bluefox_game/plugins");
}

fn main() {
    if !PLUGINS_DIR.exists() {
        std::fs::create_dir_all(PLUGINS_DIR.to_path_buf()).unwrap();
    }
    let plugins = load_plugins(PLUGINS_DIR.to_path_buf());
    for plugin in plugins {
        println!("{}", plugin.name());
    }
    internal_bluefox_lib::cleanup();
}
