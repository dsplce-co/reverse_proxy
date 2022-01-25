use clap::Parser;
use hyper::StatusCode;
use warp::{Filter, Rejection, Reply};
use warp_reverse_proxy::reverse_proxy_filter;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "http://127.0.0.1:8080/")]
    backend: String,
    #[clap(short, long, default_value_t = 4343)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let Args { backend, port } = Args::parse();

    println!("Backend: {backend}");
    println!("Listening on port {port}");

    // construct the reverse proxy
    let app = reverse_proxy_filter(
        "".to_string(),
        backend
    ).recover(bad_gateway);

    // serve it over TLS
    warp::serve(app)
        .tls()
        .cert_path("tls/ecdsa/end.fullchain")
        .key_path("tls/ecdsa/end.key")
        .run(([0, 0, 0, 0], port))
        .await;
}

async fn bad_gateway(err: Rejection) -> Result<impl Reply, Rejection> {
    eprintln!("502: {err:?}");
    let status = StatusCode::BAD_GATEWAY;
    Ok(warp::reply::with_status("couldn't connect upstream", status))
}
