use std::{
    path::PathBuf,
    process::Command,
    sync::Mutex,
};
use libloading::Library;
use tempdir::TempDir;
use lazy_static::lazy_static;

pub mod c_binds;

pub struct LibraryManager {
    libraries: Vec<Library>,
}
impl LibraryManager {
    pub fn add_lib(&mut self, l: Library) {
        self.libraries.push(l);
    }
    pub fn get_lib(&self, idx: usize) -> &Library {
        &self.libraries[idx]
    }
    pub fn len(&self) -> usize {
        self.libraries.len()
    }
}
impl Drop for LibraryManager {
    fn drop(&mut self) {
        // we are never going to use self.libraries again
        for lib in std::mem::take(&mut self.libraries) {
            lib.close().unwrap();
        }
    }
}

lazy_static! {
    static ref TEMP_DIRECTORY: TempDir = tempdir::TempDir::new("bluefox_plugins").unwrap();
    pub static ref LIBRARY_MANAGER: Mutex<Option<LibraryManager>> = Mutex::new(Some(LibraryManager { libraries: vec![] }));
}

#[repr(C)]
pub struct Plugin {
    pub name: *const i8
}
impl Plugin {
    // using any rust string type that can be converted to a c string
    pub fn new<T: AsRef<str>>(name: T) -> Self {
        Self {
            name: name.as_ref().as_ptr() as *const i8
        }
    }
    pub fn name(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(self.name).to_str().unwrap()
        }
    }
}

#[derive(Debug)]
pub enum LoadPluginError {
    CompilerNotFound(String),
    CompileError(String),
    LoadError(String),
}

pub fn load_plugins(plugin_directory: PathBuf) -> Vec<Plugin> {
    let mut out = vec![];
    for plugin_res in std::fs::read_dir(plugin_directory).unwrap() {
        if let Ok(plugin_dir) = plugin_res {
            match load_plugin(plugin_dir.path()) {
                Ok(plugin) => {
                    out.push(plugin);
                },
                Err(e) => println!("{:?}", e)
            }
        }
    }
    out
}

pub fn load_plugin(plugin_path: PathBuf) -> Result<Plugin, LoadPluginError>  {
    let mut  llc = Command::new("llc");
    let mut gcc = Command::new("gcc");
    let mut plugin_name = String::from(plugin_path.file_name().unwrap().to_str().unwrap());
    if plugin_name.contains(".bfps") {
        plugin_name = String::from(plugin_name.split_at(plugin_name.len() - 5).0);

        llc.arg("-filetype=asm").arg("-o").arg(format!("{}/{}.s", TEMP_DIRECTORY.path().to_str().unwrap(), plugin_name)).arg(format!("{}", plugin_path.to_str().unwrap()));
        if let Err(e) = llc.output() {
            return Err(LoadPluginError::CompileError(format!("bfps to asm file compilation failed: llc: {}", e)));
        }

        gcc.arg("-nostdlib").arg("-shared").arg(format!("-L{}", std::env::current_exe().unwrap().parent().unwrap().join("assets/pluginlib").to_str().unwrap())).arg("-lbluefox_lib").arg("-o").arg(format!("{}/{}.bfpn", TEMP_DIRECTORY.path().to_str().unwrap(), plugin_name)).arg("-fPIC").arg(format!("{}/{}.s", TEMP_DIRECTORY.path().to_str().unwrap(), plugin_name));
        if let Err(e) = gcc.output() {
            return Err(LoadPluginError::CompileError(format!("asm file to bfpn compilation failed: gcc: {}", e)));
        }

        // we must assume the plugin implements get_plugin and Plugin correctly, there's no way around this
        let out = unsafe {
            let mut library_manager = LIBRARY_MANAGER.lock().unwrap();
            library_manager.as_mut().unwrap().add_lib(libloading::Library::new(format!("{}/{}.bfpn", TEMP_DIRECTORY.path().to_str().unwrap(), plugin_name)).unwrap());
            let getter: libloading::Symbol<fn() -> Plugin> = library_manager.as_ref().unwrap().get_lib(library_manager.as_ref().unwrap().len() - 1).get(b"get_plugin").unwrap();
            getter()
        };
        return Ok(out);
    }
    Err(LoadPluginError::LoadError("Reached EOF before plugin was loaded".to_owned()))
}