extern crate script;

use script::{Environment, ScriptResult, StatePtr};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;


/// Checks if a file exists and is readable.
///
/// # Lua arguments
/// * `path: string`            - Path to the file to check.
fn exists(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();

    environment.state().push_bool(fs::metadata(path).is_ok());

    Ok(1)
}

/// Checks if a given path is a directory.
///
/// # Lua arguments
/// * `path: string`            - Path to check.
fn is_dir(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();

    let metadata = fs::metadata(path);
    environment.state().push_bool(metadata.is_ok() && metadata.unwrap().file_type().is_dir());

    Ok(1)
}

/// Checks if a given path is a file.
///
/// # Lua arguments
/// * `path: string`            - Path to check.
fn is_file(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();

    let metadata = fs::metadata(path);
    environment.state().push_bool(metadata.is_ok() && metadata.unwrap().file_type().is_file());

    Ok(1)
}

/// Checks if a given path is a symbolic link.
///
/// # Lua arguments
/// * `path: string`            - Path to check.
fn is_symlink(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();

    let metadata = fs::metadata(path);
    environment.state().push_bool(metadata.is_ok() && metadata.unwrap().file_type().is_symlink());

    Ok(1)
}

/// Creates a directory.
///
/// # Lua arguments
/// * `path: string`            - Path to create the directory.
fn mkdir(environment: Environment) -> ScriptResult {
    // Get the path as the first argument.
    let path = environment.state().check_string(1).to_string();

    if fs::create_dir(&path).is_err() {
        return Err(format!("failed to create directory \"{}\"", path).into());
    }

    Ok(0)
}

/// Copies a file to another location.
///
/// # Lua arguments
/// * `source: string`          - Path of the file to copy.
/// * `dest: string`            - Path to copy the file to.
fn copy(environment: Environment) -> ScriptResult {
    let source = environment.state().check_string(1).to_string();
    let dest = environment.state().check_string(2).to_string();

    if fs::copy(&source, dest).is_err() {
        return Err(format!("failed to copy \"{}\"", source).into());
    }

    Ok(0)
}

/// Moves a file from one name to another.
///
/// # Lua arguments
/// * `source: string`          - Path of the file to move.
/// * `dest: string`            - Path to move the file to.
fn rename(environment: Environment) -> ScriptResult {
    let source = environment.state().check_string(1).to_string();
    let destination = environment.state().check_string(2).to_string();

    if fs::rename(source, destination).is_err() {
        return Err("no such file or directory".into());
    }

    Ok(0)
}

/// Removes a file or empty directory.
///
/// # Lua arguments
/// * `path: string`            - Path of the file or directory to remove.
fn remove(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();

    if let Ok(metadata) = fs::metadata(&path) {
        if metadata.file_type().is_dir() {
            if fs::remove_dir_all(path).is_err() {
                return Err("failed to remove directory".into());
            }
        } else {
            if fs::remove_file(path).is_err() {
                return Err("failed to remove file".into());
            }
        }
    }

    Ok(0)
}

/// Reads an entire file and returns its contents.
///
/// # Lua arguments
/// * `path: string`            - Path of the file to read from.
fn get(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();

    let file = File::open(path);

    if file.is_err() {
        return Err("failed to open file".into());
    }

    let mut file = file.unwrap();
    let mut buffer = String::new();

    if file.read_to_string(&mut buffer).is_err() {
        return Err("failed to read file".into());
    }

    environment.state().push_string(&buffer);

    Ok(1)
}

/// Puts a string into the contents of a file.
///
/// # Lua arguments
/// * `path: string`            - Path to the file to write to.
/// * `contents: string`        - The contents to write.
fn put(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();
    let contents = String::from(environment.state().check_string(2));

    let file = OpenOptions::new()
                   .write(true)
                   .truncate(true)
                   .create(true)
                   .open(path);

    if file.is_err() {
        return Err("failed to open file".into());
    }

    let mut file = file.unwrap();
    if file.write_all(contents.as_bytes()).is_err() {
        return Err("failed to write to file".into());
    }

    Ok(0)
}

/// Appends a string to the end of the contents of a file.
///
/// # Lua arguments
/// * `path: string`            - Path to the file to append to.
/// * `contents: string`        - The contents to append.
fn append(environment: Environment) -> ScriptResult {
    let path = environment.state().check_string(1).to_string();
    let contents = String::from(environment.state().check_string(2));

    let file = OpenOptions::new()
                   .write(true)
                   .append(true)
                   .open(path);

    if file.is_err() {
        return Err("failed to open file".into());
    }

    let mut file = file.unwrap();
    if file.write_all(contents.as_bytes()).is_err() {
        return Err("failed to write to file".into());
    }

    Ok(0)
}

/// Combines the contents of two or more files into a new, single file.
///
/// # Lua arguments
/// * `sources: table`          - A list of source files to combine.
/// * `dest: string`            - The path to the output file.
fn combine(environment: Environment) -> ScriptResult {
    if !environment.state().is_table(1) {
        return Err("first argument must be a table".into());
    }

    // Open the output file for writing.
    let dest = environment.state().check_string(2).to_string();
    let out_file = OpenOptions::new()
                       .write(true)
                       .truncate(true)
                       .create(true)
                       .open(&dest);

    if out_file.is_err() {
        return Err(format!("failed to open file \"{}\"", dest).into());
    }

    let mut out_file = out_file.unwrap();

    // Walk through each path in the sources table and write their contents.
    for mut item in environment.iter(1) {
        let source: String = item.value().unwrap();

        let in_file = File::open(&source);
        if in_file.is_err() {
            return Err(format!("failed to open file \"{}\"", source).into());
        }

        // Read the source file's contents.
        let mut in_file = in_file.unwrap();
        let mut buffer = String::new();

        if in_file.read_to_string(&mut buffer).is_err() {
            return Err(format!("failed to read file \"{}\"", source).into());
        }

        // Write the source file contents into the output file.
        if out_file.write_all(buffer.as_bytes()).is_err() {
            return Err(format!("failed to write to file \"{}\"", dest).into());
        }
    }

    Ok(0)
}


#[no_mangle]
pub unsafe extern fn luaopen_fs(ptr: StatePtr) -> i32 {
    let environment = Environment::from_ptr(ptr);

    environment.register_lib(&[
        ("exists", exists),
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
        ("combine", combine)
    ]);
    1
}
