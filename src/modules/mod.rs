use runtime::{Module, Runtime, RuntimePtr};

pub mod fs;


/// Fetches the source of a built-in Lua module by a given name.
///
/// Returns an `Option<&str>` containing the Lua source of the given module, or `None` if the
/// module is not defined.
///
/// The list of available built-in modules is determined at compile-time and is embedded into the
/// Rote binary itself.
pub fn fetch(name: &str) -> Option<Module> {
    match name {
        // Statically include and match the built-in modules.
        "core"      => Some(Module::Builtin(include_str!("core.lua"))),
        "cargo"     => Some(Module::Builtin(include_str!("cargo.lua"))),
        "cpp"       => Some(Module::Builtin(include_str!("cpp.lua"))),
        "docker"    => Some(Module::Builtin(include_str!("docker.lua"))),
        "fs"        => Some(Module::Native(fs::MTABLE)),
        "git"       => Some(Module::Builtin(include_str!("git.lua"))),
        "java"      => Some(Module::Builtin(include_str!("java.lua"))),
        "php"       => Some(Module::Builtin(include_str!("php.lua"))),
        "table"     => Some(Module::Builtin(include_str!("table.lua"))),
        // If you want to add a built-in module, add your module name and file here.
        _ => None
    }
}

/// A Lua module loader that loads built-in modules.
///
/// # Lua arguments
/// * `name: string`         - The name of the module to load.
pub fn loader<'r>(runtime: RuntimePtr) -> i32 {
    // Get the module name as the first argument.
    let name = Runtime::borrow(runtime).state.check_string(1);

    if let Some(module) = fetch(name) {
        match module {
            Module::Builtin(source) => {
                Runtime::borrow(runtime).state.load_string(source);
            },
            Module::Native(_) => {
                Runtime::borrow(runtime).push_fn(loader_native);
            },
        };
    } else {
        Runtime::borrow(runtime).state.push_string(&format!("\n\tno builtin module '{}'", name));
    }
    1
}

/// Native module loader callback.
fn loader_native<'r>(runtime: RuntimePtr) -> i32 {
    let name = Runtime::borrow(runtime).state.check_string(1);

    if let Some(Module::Native(mtable)) = fetch(name) {
        Runtime::borrow(runtime).state.new_table();

        for &(name, func) in mtable.0 {
            Runtime::borrow(runtime).push_fn(func);
            Runtime::borrow(runtime).state.set_field(-2, name);
        }
        1
    } else {
        0
    }
}
