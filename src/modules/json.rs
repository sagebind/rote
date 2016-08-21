use json::{self, JsonValue};
use lua;
use runtime::{Runtime, ScriptResult};
use std::error::Error;


fn parse(runtime: Runtime) -> ScriptResult {
    fn push_value(runtime: &Runtime, value: &JsonValue) {
        match value {
            &JsonValue::Null => {
                runtime.state().push_nil();
            },
            &JsonValue::Short(_) | &JsonValue::String(_) => {
                runtime.state().push_string(value.as_str().unwrap());
            },
            &JsonValue::Number(_) => {
                runtime.state().push_number(value.as_f64().unwrap());
            },
            &JsonValue::Boolean(value) => {
                runtime.state().push_bool(value);
            },
            &JsonValue::Object(_) => {
                runtime.state().new_table();

                for (key, value) in value.entries() {
                    runtime.state().push_string(key);
                    push_value(runtime, value);
                    runtime.state().set_table(-3);
                }
            },
            &JsonValue::Array(_) => {
                runtime.state().new_table();

                let mut index = 1;
                for value in value.members() {
                    runtime.state().push_number(index as f64);
                    push_value(runtime, value);
                    runtime.state().set_table(-3);

                    index += 1;
                }
            },
        }
    }

    let source = runtime.state().check_string(1).to_string();
    let value = try!(json::parse(&source));
    push_value(&runtime, &value);

    Ok(1)
}

fn stringify(runtime: Runtime) -> ScriptResult {
    fn to_json(runtime: &Runtime, index: i32) -> Result<JsonValue, Box<Error>> {
        let lua_type = runtime.state().type_of(index);

        match lua_type {
            Some(lua::Type::Nil) | None => Ok(JsonValue::Null),
            Some(lua::Type::Boolean) => Ok(runtime.state().to_bool(index).into()),
            Some(lua::Type::Number) => Ok(runtime.state().to_number(index).into()),
            Some(lua::Type::String) => Ok(runtime.state().to_str_in_place(index).into()),
            Some(lua::Type::Table) => {
                // If the table contains only sequential numeric keys, we need to create an array instead. To do this
                // in one pass, we will fill up an object and an array simultaneously, then determine which one to
                // return at the end.
                let mut object = JsonValue::new_object();
                let mut array = JsonValue::new_array();
                let mut is_array = true;
                let mut array_index = 1;

                for (key, value) in runtime.iter(index) {
                    // Check for sequential numeric keys.
                    if !runtime.state().is_number(key) || runtime.state().to_number(key) as i32 != array_index {
                        is_array = false;
                    }

                    let value = try!(to_json(runtime, value));

                    // Insert into the array if there is still hope for it being an indexed table.
                    if is_array {
                        try!(array.push(value.clone()));
                        array_index += 1;
                    }

                    // Insert into the object.
                    let key = runtime.state().to_str(key).unwrap().to_string();
                    runtime.state().pop(1);
                    object[key] = value;
                }

                Ok(if is_array {
                    array
                } else {
                    object
                })
            },
            _ => {
                Err(format!("cannot convert {} to JSON", runtime.state().typename_of(lua_type.unwrap_or(lua::Type::None))).into())
            },
        }
    }

    let value = try!(to_json(&runtime, 1));
    let string = if runtime.state().to_bool(2) {
        let spaces = if runtime.state().get_top() >= 3 {
            runtime.state().check_number(3) as u16
        } else {
            4
        };

        json::stringify_pretty(value, spaces)
    } else {
        json::stringify(value)
    };
    runtime.state().push_string(&string);

    Ok(1)
}

pub fn load(runtime: Runtime) -> ScriptResult {
    runtime.load_lib(&[
        ("parse", parse),
        ("stringify", stringify),
    ]);

    Ok(1)
}
