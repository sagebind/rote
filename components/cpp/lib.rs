extern crate gcc;
extern crate runtime;

use runtime::{Runtime, RuntimeResult, StatePtr};


fn create_binary(runtime: Runtime) -> RuntimeResult {
    let mut config = gcc::Config::new();
    config.cargo_metadata(false);

    let mut cmd = config.get_compiler().to_command();
    cmd.arg("main.cpp");
    assert!(cmd.status().unwrap().success());

    Ok(0)
}


#[no_mangle]
pub unsafe extern fn luaopen_cpp(ptr: StatePtr) -> i32 {
    let mut runtime = Runtime::from_ptr(ptr);

    runtime.register_lib(&[
        ("binary", create_binary),
    ]);

    1
}
