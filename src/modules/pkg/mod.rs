use modules::ModuleTable;
use modules::pkg::deb::PackageBuilder;
use runtime::Runtime;
use std::error::Error;
use std::fs::File;
use tar::Archive;
use lua;

mod deb;


pub const MTABLE: ModuleTable = ModuleTable(&[
    ("tar", tar),
    ("deb", deb),
]);


fn tar(runtime: &mut Runtime, _: Option<usize>) -> i32 {
    let path = runtime.state().check_string(1).to_string();
    runtime.state().check_type(2, lua::Type::Table);

    let file = File::create(path).unwrap();
    let archive = Archive::new(file);

    runtime.state().push_nil();
    while runtime.state().next(2) {
        let input_file = runtime.state().to_str(-1).unwrap().to_string();
        runtime.state().pop(2);

        if archive.append_path(input_file).is_err() {
            runtime.throw_error("failed to create tar archive");
            return 0;
        }
    }

    if archive.finish().is_err() {
        runtime.throw_error("failed to create tar archive");
    }

    0
}

fn deb(runtime: &mut Runtime, _: Option<usize>) -> i32 {
    runtime.state().check_type(1, lua::Type::Table);

    let mut builder = PackageBuilder::new();
    let mut file: Option<String> = None;

    runtime.state().push_nil();
    while runtime.state().next(1) {
        match runtime.state().to_str(-2).unwrap() {
            "file" => {
                file = Some(runtime.state().check_string(-2).to_string());
            }
            "name" => {
                builder.name(runtime.state().check_string(-2));
            }
            "section" => {
                builder.section(runtime.state().check_string(-2));
            }
            "description" => {
                builder.short_desc(runtime.state().check_string(-2));
            }
            "size" => {
                builder.size(runtime.state().check_integer(-2) as u64);
            }
            "version" => {
                builder.version(runtime.state().check_string(-2));
            }
            "maintainer" => {
                builder.maintainer(runtime.state().check_string(-2));
            }
            "homepage" => {
                builder.homepage(runtime.state().check_string(-2));
            }
            _ => {}
        }

        runtime.state().pop(2);
    }

    if file.is_none() {
        runtime.throw_error("no file name specified");
        return 0;
    }

    let package = builder.build();
    if let Err(error) = package {
        runtime.throw_error(error.description());
        return 0;
    }
    let package = package.unwrap();

    package.write_to(&file.unwrap());

    0
}
