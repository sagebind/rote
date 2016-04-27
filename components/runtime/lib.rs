extern crate libc;
pub extern crate lua;
mod iter;
mod runtime;

pub use iter::{TableIterator, TableItem};
pub use runtime::{Function, Runtime, RuntimeResult};

// Convenience alias for Lua state pointers.
pub type StatePtr = *mut lua::ffi::lua_State;
