extern crate runtime;

use runtime::{LuaState, Runtime};


#[no_mangle]
pub extern fn luaopen_sass(ptr: *mut LuaState) -> i32 {
    let mut runtime = Runtime::from_ptr(ptr);
    runtime.eval("print(\"Sass loaded! ;)\")").unwrap();
    0
}
