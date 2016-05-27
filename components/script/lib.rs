extern crate filetime;
extern crate libc;
#[macro_use] extern crate log;
pub extern crate lua;

mod environment;
mod iter;
pub mod rule;
pub mod task;

pub use environment::Environment;
pub use iter::{TableIterator, TableItem};

/// Results that are returned by functions callable from Lua.
pub type ScriptResult = std::result::Result<i32, Box<std::error::Error>>;

// Convenience alias for Lua state pointers.
pub type StatePtr = *mut lua::ffi::lua_State;
