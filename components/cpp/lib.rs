extern crate script;

use script::{Environment, ScriptResult, StatePtr};
use script::lua;
use script::rule::Rule;
use std::process::Command;
use std::rc::Rc;


fn create_binary(environment: Environment) -> ScriptResult {
    // We expect a single table of options.
    environment.state().check_type(1, lua::Type::Table);

    let mut name = None;
    let mut build_dir = "dist".to_string();
    let mut debug = false;
    let mut compiler_flags = vec!["-Wall"];
    let mut compiler = "g++";
    let mut opt_level: u8 = 0;
    let mut files = Vec::new();

    for mut item in environment.iter(1) {
        match &item.key::<String>().unwrap() as &str {
            "name" => {
                name = Some(item.value::<String>().unwrap())
            }
            "build_dir" => {
                build_dir = item.value().unwrap()
            }
            "debug" => {
                debug = item.value().unwrap()
            }
            "src" => {
                for mut item in environment.iter(-1) {
                    let file: String = item.value().unwrap();
                    files.push(file);
                }
            }
            _ => {}
        }
    }

    if name.is_none() {
        return Err("name must be specified".into());
    }
    let name = name.unwrap();

    // Rule for individual files
    for file in &files {
        let cflags = compiler_flags.clone();
        let e = environment.clone();

        environment.create_rule(Rc::new(Rule::new(file.clone(), Vec::new(), Some(Rc::new(move |output| {
            let compiler = e.var("CC").unwrap_or(compiler.to_string());
            let mut command = Command::new(&compiler);
            command.args(&cflags);

            // Set optimization level
            command.arg(format!("-O{}", opt_level));

            // Debug symbols
            if debug {
                command.arg("-g");
            }

            // Output file
            command.arg("-c");
            command.arg("-o");
            command.arg(format!("{}.o", &output));
            command.arg(&output);

            println!("{:?}", command);

            Ok(())
        })))));
    }

    // Rule for final binary
    let e = environment.clone();
    environment.create_rule(Rc::new(Rule::new(name, files.clone(), Some(Rc::new(move |output| {
        let compiler = e.var("CC").unwrap_or(compiler.to_string());
        let mut command = Command::new(&compiler);

        // Set optimization level
        command.arg(format!("-O{}", opt_level));

        // Debug symbols
        if debug {
            command.arg("-g");
        }

        // Output file
        command.arg("-o");
        command.arg(&output);

        println!("{:?}", command);

        Ok(())
    })))));

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
