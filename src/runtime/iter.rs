use lua;
use std::any::Any;


/// An iterator for looping over the keys and values in a Lua table on the stack.
pub struct TableIterator {
    index: lua::Index,
    state: lua::State,
}

impl TableIterator {
    /// Creates a new iterator for a given table on the stack.
    pub fn new(mut state: lua::State, index: lua::Index) -> TableIterator {
        // Push an initial key for lua_next().
        state.push_nil();

        TableIterator {
            // Normalize the index.
            index: if index < 0 {
                state.get_top() + index
            } else {
                index
            },
            state: state,
        }
    }
}

impl Iterator for TableIterator {
    type Item = TableItem;

    /// Fetches the next key/value pair in the table.
    fn next(&mut self) -> Option<TableItem> {
        if self.state.next(self.index) {
            return Some(TableItem {
                state: unsafe {
                    lua::State::from_ptr(self.state.as_ptr())
                },
            });
        }

        None
    }
}

/// Represents a key/value pair in a table.
pub struct TableItem {
    state: lua::State,
}

impl TableItem {
    /// Gets the item key as a given type.
    pub fn key<T : Any + lua::FromLua>(&mut self) -> Option<T> {
        self.state.to_type::<T>(-2).map(|key| {
            if let Some(_) = (&key as &Any).downcast_ref::<String>() {
                self.state.pop(1);
            }

            key
        })
    }

    /// Gets the item value as a given type.
    pub fn value<T : Any + lua::FromLua>(&mut self) -> Option<T> {
        self.state.to_type::<T>(-1).map(|key| {
            if let Some(_) = (&key as &Any).downcast_ref::<String>() {
                self.state.pop(1);
            }

            key
        })
    }
}

impl Drop for TableItem {
    fn drop(&mut self) {
        self.state.pop(1);
    }
}
