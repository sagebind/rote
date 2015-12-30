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

    for item in runtime.iter(2) {
        let input_file: String = item.value().unwrap();

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

    for item in runtime.iter(1) {
        match &item.key::<String>().unwrap() as &str {
            "output" => {
                file = Some(item.value().unwrap());
            }
            "name" => {
                builder.name(&item.value::<String>().unwrap());
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
            "size" => {
                builder.size(item.value::<lua::Integer>().unwrap() as u64);
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
            _ => {}
        }
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
