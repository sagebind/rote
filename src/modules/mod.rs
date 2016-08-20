use runtime::Runtime;

pub mod cpp;
pub mod http;
pub mod fs;
pub mod java;
pub mod json;
pub mod stdlib;


pub fn register_all(runtime: &Runtime) {
    self::stdlib::load(runtime.clone());
    runtime.register_lib("cpp", self::cpp::load);
    runtime.register_lib("http", self::http::load);
    runtime.register_lib("fs", self::fs::load);
    runtime.register_lib("java", self::java::load);
    runtime.register_lib("json", self::json::load);
}
