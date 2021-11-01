use std::env;
use hyper::StatusCode;
use warp::{Filter, Rejection, Reply};
use warp_reverse_proxy::reverse_proxy_filter;

#[tokio::main]
async fn main() {
    // defaults
    let mut backend = "http://127.0.0.1:8080/".to_string();
    let mut port = 4343;

    // pass backend URL as a command line argument
    if let Some(arg) = env::args().skip(1).next() {
        backend = arg;
    }

    // check the PORT env var
    if let Ok(env_str) = env::var("PORT") {
        if let Ok(new_port) = env_str.parse::<u16>() {
            port = new_port;
        }
    }

    println!("Backend: {}", backend);
    println!("Listening on port {}", port);

    // construct the reverse proxy
    let app = reverse_proxy_filter(
        "".to_string(),
        backend
    ).recover(bad_gateway);

    // serve it over TLS
    warp::serve(app)
        .tls()
        .cert_path("tls/ecdsa/client.fullchain")
        .key_path("tls/ecdsa/client.key")
        .run(([0, 0, 0, 0], port))
        .await;
}

async fn bad_gateway(err: Rejection) -> Result<impl Reply, Rejection> {
    eprintln!("502: {:?}", err);
    let status = StatusCode::BAD_GATEWAY;
    Ok(warp::reply::with_status("couldn't connect upstream", status))
}
