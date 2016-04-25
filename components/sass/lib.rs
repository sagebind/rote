extern crate runtime;
extern crate sass_rs;

use runtime::{Runtime, RuntimeResult, StatePtr};
use sass_rs::sass_context::{OutputStyle, SassFileContext};


fn compile(runtime: Runtime) -> RuntimeResult {
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
    let mut runtime = Runtime::from_ptr(ptr);

    runtime.register_lib(&[
        ("compile", compile)
    ]);

    1
}
