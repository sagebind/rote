extern crate flate2;
extern crate runtime;
extern crate tar;
extern crate time;
mod ar;
mod deb;

use deb::*;
use runtime::{Runtime, RuntimeResult, StatePtr};
use runtime::lua;
use std::fs::File;
use std::path::Path;
use tar::Archive;


fn tar(mut runtime: Runtime) -> RuntimeResult {
    let path = runtime.state().check_string(1).to_string();
    runtime.state().check_type(2, lua::Type::Table);

    let file = File::create(path).unwrap();
    let archive = Archive::new(file);

    for item in runtime.iter(2) {
        let input_file: String = item.value().unwrap();

        if archive.append_path(input_file).is_err() {
            return Err("failed to create tar archive".into());
        }
    }

    if archive.finish().is_err() {
        return Err("failed to create tar archive".into());
    }

    Ok(0)
}

fn deb(mut runtime: Runtime) -> RuntimeResult {
    runtime.state().check_type(1, lua::Type::Table);

    let mut builder = PackageBuilder::new();
    let mut file: Option<String> = None;

    for item in runtime.iter(1) {
        match &item.key::<String>().unwrap() as &str {
            "output" => {
                file = Some(item.value().unwrap());
            }
            "name" => {
                builder.name(&item.value::<String>().unwrap());
            }
            "priority" => {
                match &item.value::<String>().unwrap().to_lowercase() as &str {
                    "essential" => {
                        builder.priority(Priority::Essential);
                    }
                    "extra" => {
                        builder.priority(Priority::Extra);
                    }
                    "important" => {
                        builder.priority(Priority::Important);
                    }
                    "optional" => {
                        builder.priority(Priority::Optional);
                    }
                    "required" => {
                        builder.priority(Priority::Required);
                    }
                    "standard" => {
                        builder.priority(Priority::Standard);
                    }
                    _ => {}
                }
            }
            "arch" => {
                match &item.value::<String>().unwrap().to_lowercase() as &str {
                    "all" => {
                        builder.arch(Arch::All);
                    }
                    "x86" => {
                        builder.arch(Arch::X86);
                    }
                    "i386" => {
                        builder.arch(Arch::X86);
                    }
                    "x64" => {
                        builder.arch(Arch::X64);
                    }
                    "x86_64" => {
                        builder.arch(Arch::X64);
                    }
                    "amd64" => {
                        builder.arch(Arch::X64);
                    }
                    _ => {}
                }
            }
            "section" => {
                builder.section(&item.value::<String>().unwrap());
            }
            "depends" => {
                for item in runtime.iter(-1) {
                    let package: String = item.key().unwrap();
                    let version: String = item.value().unwrap();

                    builder.add_depends((&package, &version));
                }
            }
            "description" => {
                builder.short_desc(&item.value::<String>().unwrap());
            }
            "long_description" => {
                builder.long_desc(&item.value::<String>().unwrap());
            }
            "version" => {
                builder.version(&item.value::<String>().unwrap());
            }
            "maintainer" => {
                builder.maintainer(&item.value::<String>().unwrap());
            }
            "homepage" => {
                builder.homepage(&item.value::<String>().unwrap());
            }
            "files" => {
                for item in runtime.iter(-1) {
                    let dest: String = item.key().unwrap();
                    let source: String = item.value().unwrap();

                    builder.add_file(Path::new(&source), Path::new(&dest));
                }
            }
            _ => {}
        }
    }

    if file.is_none() {
        return Err("no file name specified".into());
    }

    let mut file = File::create(file.unwrap()).unwrap();

    let package = try!(builder.build());
    package.write_to(&mut file);

    Ok(0)
}


#[no_mangle]
pub unsafe extern fn luaopen_pkg(ptr: StatePtr) -> i32 {
    let mut runtime = Runtime::from_ptr(ptr);
    runtime.register_lib(&[
        ("tar", tar),
        ("deb", deb),
    ]);
    1
}
