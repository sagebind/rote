extern crate script;
extern crate solicit;

use script::{Environment, ScriptResult, StatePtr};
use solicit::http::client::CleartextConnector;
use solicit::client::SimpleClient;


/// Downloads a file.
fn download(environment: Environment) -> ScriptResult {
    let url = environment.state().check_string(1).to_string();

    let connector = CleartextConnector::new("http2bin.org");
    let mut client = SimpleClient::with_connector(connector).unwrap();
    let response = client.get(b"/get", &[]).unwrap();

    if let Ok(status) = response.status_code() {
        if status != 200 {}
    }

    Ok(0)
}


#[no_mangle]
pub unsafe extern fn luaopen_http(ptr: StatePtr) -> i32 {
    let environment = Environment::from_ptr(ptr);

    environment.register_lib(&[
        ("download", download),
    ]);

    1
}
