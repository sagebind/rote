extern crate docker;
extern crate script;

use docker::Docker;
use script::{Environment, ScriptResult, StatePtr};


pub fn build(environment: Environment) -> ScriptResult {
    Ok(0)
}


#[no_mangle]
pub unsafe extern fn luaopen_docker(ptr: StatePtr) -> i32 {
    let mut docker = match Docker::connect("unix:///var/run/docker.sock") {
        Ok(docker) => docker,
        Err(e) => { panic!("{}", e); }
    };

    1
}
