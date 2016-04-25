extern crate runtime;
extern crate solicit;

use runtime::{Runtime, RuntimeResult, StatePtr};
use solicit::http::client::CleartextConnector;
use solicit::client::SimpleClient;


/// Downloads a file.
fn download(runtime: Runtime) -> RuntimeResult {
    let url = runtime.state().check_string(1).to_string();

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
    let mut runtime = Runtime::from_ptr(ptr);

    runtime.register_lib(&[
        ("download", download),
    ]);

    1
}
