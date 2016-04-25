extern crate docker;
extern crate runtime;

use docker::Docker;
use runtime::{Runtime, StatePtr};


pub fn build(mut runtime: Runtime) -> i32 {
    0
}


#[no_mangle]
pub unsafe extern fn luaopen_docker(ptr: StatePtr) -> i32 {
    let mut docker = match Docker::connect("unix:///var/run/docker.sock") {
        Ok(docker) => docker,
        Err(e) => { panic!("{}", e); }
    };

    1
}
