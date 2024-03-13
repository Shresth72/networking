#![allow(unused)]

use http_body_util::{BodyExt, Empty, Full};
use hyper::{
    body::{Bytes, Incoming as Body},
    client::conn::http1 as Client,
    server::conn::http1 as Server,
    service::service_fn,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{
    convert::Infallible,
    io::{self, Write},
    net::SocketAddr,
};
use tokio::net::{TcpListener, TcpStream};

type ErrorType = dyn std::error::Error + Send + Sync;

async fn log(req: Request<Body>) -> Result<Response<Body>, Box<ErrorType>> {
    // Basic Middleware (Log Path of the incoming request)

    let path = req.uri().path();

    if path.starts_with("/api") {
        println!("API Path: {}", path);
    } else {
        println!("Generic Path: {}", path);
    }

    handle(req).await
}

async fn handle(req: Request<Body>) -> Result<Response<Body>, Box<ErrorType>> {
    // Client Request Sender

    let uri = req.uri().to_string().parse::<hyper::Uri>()?;
    let host = uri.host().expect("No host in the URL");
    let port = uri.port_u16().unwrap_or(80);

    let addr: String = format!("{}:{}", host, port);

    let stream = TcpStream::connect(addr).await?;

    let io = TokioIo::new(stream);
    let (mut sender, conn) = Client::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });

    let authority = uri
        .authority()
        .expect("No authority in the URL")
        .clone()
        .to_string();

    let path = uri.path();

    // GET Request
    let req = Request::builder()
        .method(req.method())
        .uri(path)
        .header(hyper::header::HOST, authority)
        .body(req.into_body())
        .expect("Failed to build request"); // ? to return Box Error

    let mut res = sender.send_request(req).await?;

    println!("Response: {:?}", res.status());
    println!("Headers: {:?}", res.headers());

    // Stream each frame of the body to io::stdout, instead of buffer
    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            io::stdout().write_all(&chunk)?;
        }
    }

    // TODO: POST Request

    println!("Request sent successfully");
    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), Box<ErrorType>> {
    // Proxy Server

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on: {}", addr);

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
            } else {
                println!("Connection served successfully");
            }
        });
    }
}
