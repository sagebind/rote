use lua;
use runtime::Runtime;
use std::any::Any;


/// An iterator for looping over the keys and values in a Lua table on the stack.
pub struct TableIterator {
    index: lua::Index,
    runtime: Runtime,
}

impl TableIterator {
    /// Creates a new iterator for a given table on the stack.
    pub fn new(runtime: Runtime, index: lua::Index) -> TableIterator {
        // Push an initial key for lua_next().
        runtime.state().push_nil();

        TableIterator {
            // Normalize the index.
            index: if index < 0 {
                runtime.state().get_top() + index
            } else {
                index
            },
            runtime: runtime,
        }
    }
}

impl Iterator for TableIterator {
    type Item = TableItem;

    /// Fetches the next key/value pair in the table.
    fn next(&mut self) -> Option<TableItem> {
        if self.runtime.state().next(self.index) {
            return Some(TableItem {
                runtime: self.runtime.clone(),
            });
        }

        None
    }
}

/// Represents a key/value pair in a table.
pub struct TableItem {
    runtime: Runtime,
}

impl TableItem {
    /// Gets the item key as a given type.
    pub fn key<T : Any + lua::FromLua>(&self) -> Option<T> {
        self.runtime.state().to_type::<T>(-2).map(|key| {
            if let Some(_) = (&key as &Any).downcast_ref::<String>() {
                self.runtime.state().pop(1);
            }

            key
        })
    }

    /// Gets the item value as a given type.
    pub fn value<T : Any + lua::FromLua>(&self) -> Option<T> {
        self.runtime.state().to_type::<T>(-1).map(|key| {
            if let Some(_) = (&key as &Any).downcast_ref::<String>() {
                self.runtime.state().pop(1);
            }

            key
        })
    }
}

impl Drop for TableItem {
    fn drop(&mut self) {
        self.runtime.state().pop(1);
    }
}
