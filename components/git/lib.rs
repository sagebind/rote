extern crate git2;
extern crate script;

use git2::{Error, Repository};
use script::{Environment, ScriptResult, StatePtr};


fn get_repo() -> Result<Repository, Error> {
    Repository::open(".")
}

fn tag(environment: Environment) -> ScriptResult {
    Ok(0)
}

#[no_mangle]
pub unsafe extern fn luaopen_git(ptr: StatePtr) -> i32 {
    let environment = Environment::from_ptr(ptr);

    environment.register_lib(&[
        ("tag", tag),
    ]);
    1
}
