use runtime::RuntimeFn;

//mod fs;
//mod http;
//mod pkg;
mod rote;


/// A descriptor struct for a loadable module.
pub enum Module {
    /// Entry point for a native runtime module.
    Native(ModuleTable),

    /// A plain Lua module that is built-in.
    Builtin(&'static str),
}

/// An entrypoint table for a native module.
pub struct ModuleTable(pub &'static [(&'static str, RuntimeFn)]);

/// Fetches a built-in Lua module by the given module's name.
///
/// Returns an `Option<Module>` containing the Lua source or MTABLE of the given module, or `None`
/// if the module is not defined.
///
/// The list of available built-in modules is determined at compile-time and modules embedded into
/// the Rote binary itself.
pub fn fetch(name: &str) -> Option<Module> {
    match name {
        // Statically include and match the built-in modules.
        "busted"      => Some(Module::Builtin(include_str!("busted.lua"))),
        "core"      => Some(Module::Builtin(include_str!("core.lua"))),
        "cargo"     => Some(Module::Builtin(include_str!("cargo.lua"))),
        "cpp"       => Some(Module::Builtin(include_str!("cpp.lua"))),
        "docker"    => Some(Module::Builtin(include_str!("docker.lua"))),
        "dsl"       => Some(Module::Builtin(include_str!("dsl.lua"))),
        //"fs"        => Some(Module::Native(fs::MTABLE)),
        "git"       => Some(Module::Builtin(include_str!("git.lua"))),
        //"http"      => Some(Module::Native(http::MTABLE)),
        "java"      => Some(Module::Builtin(include_str!("java.lua"))),
        //"pkg"       => Some(Module::Native(pkg::MTABLE)),
        "php"       => Some(Module::Builtin(include_str!("php.lua"))),
        "rote"      => Some(Module::Native(rote::MTABLE)),
        "table"     => Some(Module::Builtin(include_str!("table.lua"))),
        // If you want to add a built-in module, add your module name and file here.
        _ => None
    }
}
