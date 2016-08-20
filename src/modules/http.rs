use hyper::client::Client;
use runtime::{Runtime, ScriptResult};
use std::io::Read;


/// Sends an HTTP GET request and returns the response.
fn get(runtime: Runtime) -> ScriptResult {
    let url = runtime.state().check_string(1).to_string();
    let client = Client::new();

    // Send the request.
    let mut response = try!(client.get(&url).send());

    // Return the response text and the status code.
    let mut body = String::new();
    try!(response.read_to_string(&mut body));
    runtime.state().push(body);
    runtime.state().push(response.status.to_u16() as f64);

    Ok(2)
}

/// Sends an HTTP POST request with a body and returns the response.
fn post(runtime: Runtime) -> ScriptResult {
    let url = runtime.state().check_string(1).to_string();
    let client = Client::new();

    // Get the request body.
    let request_body = runtime.state().to_str(2).unwrap_or("").to_string();

    // Send the request.
    let mut response = try!(client.post(&url).body(&request_body).send());

    // Return the response text and the status code.
    let mut response_body = String::new();
    try!(response.read_to_string(&mut response_body));
    runtime.state().push(response_body);
    runtime.state().push(response.status.to_u16() as f64);

    Ok(2)
}

pub fn load(runtime: Runtime) -> ScriptResult {
    runtime.load_lib(&[
        ("get", get),
        ("post", post),
    ]);

    Ok(1)
}
