pub mod plugin;

pub fn cleanup() {
    let library_manager = plugin::LIBRARY_MANAGER.lock().unwrap().take();
    drop(library_manager.unwrap())
}