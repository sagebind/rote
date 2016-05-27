extern crate gcc;
extern crate script;

use script::{Environment, ScriptResult, StatePtr};


fn create_binary(environment: Environment) -> ScriptResult {
    let mut config = gcc::Config::new();
    config.cargo_metadata(false);

    let mut cmd = config.get_compiler().to_command();
    cmd.arg("main.cpp");
    assert!(cmd.status().unwrap().success());

    Ok(0)
}


#[no_mangle]
pub unsafe extern fn luaopen_cpp(ptr: StatePtr) -> i32 {
    let environment = Environment::from_ptr(ptr);

    environment.register_lib(&[
        ("binary", create_binary),
    ]);

    1
}
