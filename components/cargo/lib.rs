extern crate runtime;

use runtime::{Runtime, StatePtr};


static LUA: &'static str = include_str!("cargo.lua");

#[no_mangle]
pub unsafe extern fn luaopen_cargo(ptr: StatePtr) -> i32 {
    let mut runtime = Runtime::from_ptr(ptr);
    runtime.eval(LUA).unwrap();
    1
}
