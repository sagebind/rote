extern crate filetime;
extern crate libc;
#[macro_use] extern crate log;
pub extern crate lua;

mod environment;
mod iter;
pub mod rule;
mod runtime;
pub mod task;

pub use environment::Environment;
pub use iter::{TableIterator, TableItem};
pub use runtime::{Function, Runtime, RuntimeResult};

// Convenience alias for Lua state pointers.
pub type StatePtr = *mut lua::ffi::lua_State;
