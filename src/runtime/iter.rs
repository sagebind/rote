use lua;


/// An iterator for looping over the keys and values in a Lua table on the stack.
pub struct TableIterator {
    index: lua::Index,
    state: lua::State,
    first: bool,
}

impl TableIterator {
    /// Creates a new iterator for a given table on the stack.
    pub fn new(mut state: lua::State, index: lua::Index) -> TableIterator {
        TableIterator {
            // Normalize the index.
            index: if index < 0 {
                state.get_top() + index
            } else {
                index
            },
            state: state,
            first: true,
        }
    }
}

impl Iterator for TableIterator {
    type Item = (i32, i32);

    /// Fetches the next key/value index pair in the table.
    fn next(&mut self) -> Option<(i32, i32)> {
        // On the first call, push an initial key for the call to lua_next().
        if self.first {
            self.state.push_nil();
            self.first = false;
        } else {
            // Pop the previous value.
            self.state.pop(1);
        }

        if self.state.next(self.index) {
            let top = self.state.get_top();
            return Some((top - 1, top));
        }

        None
    }
}
