extern crate script;

use script::{Environment, StatePtr};


static LUA: &'static str = include_str!("cargo.lua");

#[no_mangle]
pub unsafe extern fn luaopen_cargo(ptr: StatePtr) -> i32 {
    let environment = Environment::from_ptr(ptr);
    environment.eval(LUA).unwrap();
    1
}
