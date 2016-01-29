/// Module that provides various functions for working with files and the file system.

use modules::ModuleTable;
use runtime::Runtime;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;


pub const MTABLE: ModuleTable = ModuleTable(&[("exists", exists),
                                              ("is_dir", is_dir),
                                              ("is_file", is_file),
                                              ("is_symlink", is_symlink),
                                              ("mkdir", mkdir),
                                              ("copy", copy),
                                              ("rename", rename),
                                              ("remove", remove),
                                              ("get", get),
                                              ("put", put),
                                              ("append", append),
                                              ("combine", combine)]);

/// Checks if a file exists and is readable.
///
/// # Lua arguments
/// * `path: string`            - Path to the file to check.
fn exists(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();

    runtime.state().push_bool(fs::metadata(path).is_ok());

    1
}

/// Checks if a given path is a directory.
///
/// # Lua arguments
/// * `path: string`            - Path to check.
fn is_dir(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();

    let metadata = fs::metadata(path);
    runtime.state().push_bool(metadata.is_ok() && metadata.unwrap().file_type().is_dir());

    1
}

/// Checks if a given path is a file.
///
/// # Lua arguments
/// * `path: string`            - Path to check.
fn is_file(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();

    let metadata = fs::metadata(path);
    runtime.state().push_bool(metadata.is_ok() && metadata.unwrap().file_type().is_file());

    1
}

/// Checks if a given path is a symbolic link.
///
/// # Lua arguments
/// * `path: string`            - Path to check.
fn is_symlink(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();

    let metadata = fs::metadata(path);
    runtime.state().push_bool(metadata.is_ok() && metadata.unwrap().file_type().is_symlink());

    1
}

/// Creates a directory.
///
/// # Lua arguments
/// * `path: string`            - Path to create the directory.
fn mkdir(runtime: &mut Runtime) -> i32 {
    // Get the path as the first argument.
    let path = runtime.state().check_string(1).to_string();

    if fs::create_dir(&path).is_err() {
        runtime.throw_error(&format!("failed to create directory \"{}\"", path));
    }

    0
}

/// Copies a file to another location.
///
/// # Lua arguments
/// * `source: string`          - Path of the file to copy.
/// * `dest: string`            - Path to copy the file to.
fn copy(runtime: &mut Runtime) -> i32 {
    let source = runtime.state().check_string(1).to_string();
    let dest = runtime.state().check_string(2).to_string();

    if fs::copy(&source, dest).is_err() {
        runtime.throw_error(&format!("failed to copy \"{}\"", source));
    }

    0
}

/// Moves a file from one name to another.
///
/// # Lua arguments
/// * `source: string`          - Path of the file to move.
/// * `dest: string`            - Path to move the file to.
fn rename(runtime: &mut Runtime) -> i32 {
    let source = runtime.state().check_string(1).to_string();
    let destination = runtime.state().check_string(2).to_string();

    if fs::rename(source, destination).is_err() {
        runtime.throw_error("no such file or directory");
    }

    0
}

/// Removes a file or empty directory.
///
/// # Lua arguments
/// * `path: string`            - Path of the file or directory to remove.
fn remove(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();

    if let Ok(metadata) = fs::metadata(&path) {
        if metadata.file_type().is_dir() {
            if fs::remove_dir_all(path).is_err() {
                runtime.throw_error("failed to remove directory");
            }
        } else {
            if fs::remove_file(path).is_err() {
                runtime.throw_error("failed to remove file");
            }
        }
    }

    0
}

/// Reads an entire file and returns its contents.
///
/// # Lua arguments
/// * `path: string`            - Path of the file to read from.
fn get(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();

    let file = File::open(path);

    if file.is_err() {
        runtime.throw_error("failed to open file");
        return 0;
    }

    let mut file = file.unwrap();
    let mut buffer = String::new();

    if file.read_to_string(&mut buffer).is_err() {
        runtime.throw_error("failed to read file");
        return 0;
    }

    runtime.state().push_string(&buffer);

    1
}

/// Puts a string into the contents of a file.
///
/// # Lua arguments
/// * `path: string`            - Path to the file to write to.
/// * `contents: string`        - The contents to write.
fn put(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();
    let contents = String::from(runtime.state().check_string(2));

    let file = OpenOptions::new()
                   .write(true)
                   .truncate(true)
                   .create(true)
                   .open(path);

    if file.is_err() {
        runtime.throw_error("failed to open file");
        return 0;
    }

    let mut file = file.unwrap();
    if file.write_all(contents.as_bytes()).is_err() {
        runtime.throw_error("failed to write to file");
    }

    0
}

/// Appends a string to the end of the contents of a file.
///
/// # Lua arguments
/// * `path: string`            - Path to the file to append to.
/// * `contents: string`        - The contents to append.
fn append(runtime: &mut Runtime) -> i32 {
    let path = runtime.state().check_string(1).to_string();
    let contents = String::from(runtime.state().check_string(2));

    let file = OpenOptions::new()
                   .write(true)
                   .append(true)
                   .open(path);

    if file.is_err() {
        runtime.throw_error("failed to open file");
        return 0;
    }

    let mut file = file.unwrap();
    if file.write_all(contents.as_bytes()).is_err() {
        runtime.throw_error("failed to write to file");
    }

    0
}

/// Combines the contents of two or more files into a new, single file.
///
/// # Lua arguments
/// * `sources: table`          - A list of source files to combine.
/// * `dest: string`            - The path to the output file.
fn combine(runtime: &mut Runtime) -> i32 {
    if !runtime.state().is_table(1) {
        runtime.throw_error("first argument must be a table");
        return 0;
    }

    // Open the output file for writing.
    let dest = runtime.state().check_string(2).to_string();
    let out_file = OpenOptions::new()
                       .write(true)
                       .truncate(true)
                       .create(true)
                       .open(&dest);

    if out_file.is_err() {
        runtime.throw_error(&format!("failed to open file \"{}\"", dest));
        return 0;
    }

    let mut out_file = out_file.unwrap();

    // Walk through each path in the sources table and write their contents.
    for item in runtime.iter(1) {
        let source: String = item.value().unwrap();

        let in_file = File::open(&source);
        if in_file.is_err() {
            runtime.throw_error(&format!("failed to open file \"{}\"", source));
            return 0;
        }

        // Read the source file's contents.
        let mut in_file = in_file.unwrap();
        let mut buffer = String::new();

        if in_file.read_to_string(&mut buffer).is_err() {
            runtime.throw_error(&format!("failed to read file \"{}\"", source));
            return 0;
        }

        // Write the source file contents into the output file.
        if out_file.write_all(buffer.as_bytes()).is_err() {
            runtime.throw_error(&format!("failed to write to file \"{}\"", dest));
            return 0;
        }
    }

    0
}
