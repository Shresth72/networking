use http_body_util::{Empty, Full};
use hyper::{
    body::{Bytes, Incoming as Body},
    client::conn::http1 as Client,
    server::conn::http1 as Server,
    service::service_fn,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::{TcpListener, TcpStream};

type ErrorType = dyn std::error::Error + Send + Sync;

async fn hello(_: Request<Body>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}

async fn log(req: Request<Body>) -> Result<Response<Body>, Box<ErrorType>> {
    let path = req.uri().path();

    if path.starts_with("/api") {
        println!("API Path: {}", path);
    } else {
        println!("Generic Path: {}", path);
    }

    handle(req).await
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, Box<ErrorType>> {
    // let client = Client::new();
    // client.request(req).await

    let uri = req.uri().to_string().parse::<hyper::Uri>()?;
    let host = uri.host().expect("No host in the URL");
    let port = uri.port_u16().unwrap_or(80);

    let addr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid address");

    let stream = TcpStream::connect(addr).await?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = Client::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    sender.send_request(req).await?;

    Ok()
}

#[tokio::main]
async fn main() -> Result<(), Box<ErrorType>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    // Loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // IO Trait for the server conn
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // service_fn converts our function into a Service, that can be then passed onto the http1 conn of server
            if let Err(err) = Server::Builder::new()
                .serve_connection(io, service_fn(log))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}
