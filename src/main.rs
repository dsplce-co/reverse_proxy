use clap::Parser;
use warp::{Filter, Rejection, Reply};
use warp::http::HeaderMap;
use warp::ws::{WebSocket};
use warp_reverse_proxy::reverse_proxy_filter;
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use hyper::StatusCode;

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

    let backend_ws = backend.replace("http", "ws");

    // --- WebSocket route ---
    let ws_route = warp::any()
        .and(warp::header::headers_cloned())
        .and(warp::ws())
        .and_then(move |headers: HeaderMap, ws: warp::ws::Ws| {
            let backend_ws = backend_ws.clone();
            async move {
                if is_websocket(&headers) {
                    Ok::<_, Rejection>(ws.on_upgrade(move |socket| handle_ws(socket, backend_ws)))
                } else {
                    Err(warp::reject()) // not a websocket, let HTTP route handle it
                }
            }
        });

    // --- HTTP route ---
    let http_route = reverse_proxy_filter("".to_string(), backend.clone())
        .recover(bad_gateway);

    // --- Combined ---
    let routes = ws_route.or(http_route);

    warp::serve(routes)
        .tls()
        .cert_path("tls/ecdsa/end.fullchain")
        .key_path("tls/ecdsa/end.key")
        .run(([0, 0, 0, 0], port))
        .await;
}

fn is_websocket(headers: &HeaderMap) -> bool {
    headers.get("upgrade")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

async fn handle_ws(ws: WebSocket, backend_url: String) {
    let (mut client_tx, mut client_rx) = ws.split();

    match connect_async(backend_url).await {
        Ok((backend_ws, _)) => {
            let (mut backend_tx, mut backend_rx) = backend_ws.split();

            let to_backend = async {
                while let Some(Ok(msg)) = client_rx.next().await {
                    let converted = if msg.is_text() {
                        match msg.to_str() {
                            Ok(text) => tungstenite::Message::Text(text.to_string()),
                            Err(_) => return, // Ignore or handle invalid UTF-8 in text message
                        }
                    } else if msg.is_binary() {
                        tungstenite::Message::Binary(msg.into_bytes())
                    } else if msg.is_ping() {
                        tungstenite::Message::Ping(msg.into_bytes())
                    } else if msg.is_pong() {
                        tungstenite::Message::Pong(msg.into_bytes())
                    } else if msg.is_close() {
                        let frame = msg.close_frame()
                            .map(|(code, reason)| tungstenite::protocol::CloseFrame {
                                code: code.into(),
                                reason: reason.to_string().into(),
                            });

                        tungstenite::Message::Close(frame)
                    } else {
                        eprintln!("Unknown message type: {:#?}", msg);
                        return;
                    };

                    if backend_tx.send(converted).await.is_err() {
                        break;
                    }
                }
            };

            let to_client = async {
                while let Some(Ok(msg)) = backend_rx.next().await {
                    let converted = match msg {
                        tungstenite::Message::Text(text) => warp::ws::Message::text(text),
                        tungstenite::Message::Binary(bin) => warp::ws::Message::binary(bin),
                        tungstenite::Message::Ping(_) | tungstenite::Message::Pong(_) => continue,
                        tungstenite::Message::Close(frame) => warp::ws::Message::close(),
                    };

                    if client_tx.send(converted).await.is_err() {
                        break;
                    }
                }
            };

            tokio::select! {
                _ = to_backend => (),
                _ = to_client => (),
            }
        }
        Err(e) => {
            eprintln!("Failed to connect to backend WS: {e}");
        }
    }
}

async fn bad_gateway(err: Rejection) -> Result<impl Reply, Rejection> {
    eprintln!("502: {err:?}");
    Ok(warp::reply::with_status("couldn't connect upstream", StatusCode::BAD_GATEWAY))
}