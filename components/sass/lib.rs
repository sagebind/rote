extern crate sass_rs;
extern crate script;

use sass_rs::sass_context::{OutputStyle, SassFileContext};
use script::{Environment, ScriptResult, StatePtr};


fn compile(environment: Environment) -> ScriptResult {
    let style = OutputStyle::Compressed;

    let mut file_context = SassFileContext::new("filename");
    let out = file_context.compile();
    match out {
        Ok(css) => println!("------- css ({:?}) ------\n{}--------",
                            style, css),
        Err(err) => println!("{}", err)
    };

    Ok(0)
}


#[no_mangle]
pub unsafe extern fn luaopen_sass(ptr: StatePtr) -> i32 {
    let environment = Environment::from_ptr(ptr);

    environment.register_lib(&[
        ("compile", compile)
    ]);

    1
}
