use darpi::futures::{SinkExt, StreamExt};
use darpi::{app, handler, job::FutureJob, response::UpgradeWS, Body, Method, Request};
use tokio_tungstenite::{tungstenite::protocol::Role, WebSocketStream};

#[handler]
async fn hello_world(#[request] r: Request<Body>) -> Result<UpgradeWS, String> {
    let resp = UpgradeWS::from_header(r.headers());

    FutureJob::from(async move {
        let upgraded = darpi::upgrade::on(r).await.unwrap();
        let mut ws_stream = WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await;

        while let Some(msg) = ws_stream.next().await {
            let msg = msg.unwrap();

            if msg.is_text() || msg.is_binary() {
                println!("received a message `{}`", msg);
                ws_stream.send(msg).await.unwrap();
            } else if msg.is_close() {
                println!("closing websocket");
                return;
            }
        }
    })
    .spawn()
    .map_err(|e| format!("{}", e))?;

    Ok(resp.unwrap())
}

#[tokio::main]
async fn main() -> Result<(), darpi::Error> {
    app!({
        address: "127.0.0.1:3000",
        handlers: [{
            route: "/",
            method: Method::GET,
            handler: hello_world
        }]
    })
    .run()
    .await
}
