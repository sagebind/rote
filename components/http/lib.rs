extern crate solicit;

use modules::ModuleTable;
use runtime::Runtime;
use solicit::http::client::CleartextConnector;
use solicit::client::SimpleClient;


pub const MTABLE: ModuleTable = ModuleTable(&[
    ("get", get)
]);

fn get(runtime: &mut Runtime) -> i32 {
    let url = runtime.state().check_string(1).to_string();

    let connector = CleartextConnector::new("http2bin.org");
    let mut client = SimpleClient::with_connector(connector).unwrap();
    let response = client.get(b"/get", &[]).unwrap();

    if let Ok(status) = response.status_code() {
        if status != 200 {}
    }

    0
}
