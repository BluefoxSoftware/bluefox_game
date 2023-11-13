use super::Plugin;

#[no_mangle]
pub fn new_plugin(name: *const i8) -> Plugin {
    Plugin {
        name
    }
}