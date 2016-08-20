use runtime::{Runtime, ScriptResult};

const SOURCE: &'static str = include_str!("java.lua");


/// Module loader.
pub fn load(runtime: Runtime) -> ScriptResult {
    try!(runtime.eval(SOURCE));

    Ok(1)
}
