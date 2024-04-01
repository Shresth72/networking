// #![allow(unused)]
use crate::{connection, ErrorType};

use hyper::{
    body::Incoming as Body, client::conn::http1 as Client, server::conn::http1 as Server,
    service::service_fn, Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{
    io::{self, Write},
    net::SocketAddr,
    sync::Arc,
};
use tokio::net::{TcpListener, TcpStream};

pub async fn handle(req: Request<Body>) -> Result<Response<Body>, Box<ErrorType>> {
    // Client Request Sender

    // TODO: PSbindDN and PSbindPW binding to the service servers

    let uri = req.uri().to_string().parse::<hyper::Uri>()?;
    let host = uri.host().expect("No host in the URL");
    let port = uri.port_u16().unwrap_or(80);
    let method = req.method().clone();

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

    let cache = connection::conn().await?;
    // if req.method() == hyper::Method::GET {
    //     // Check if the request is already cached
    //     let cache = Arc::clone(&cache);

    //     println!("Cache Miss");
    // }

    // Processing the request
    let req = Request::builder()
        .method(req.method())
        .uri(path)
        .header(hyper::header::HOST, authority)
        .body(req.into_body())
        .expect("Failed to build request");

    // println!("Request: {:?}", req);

    // cache set
    cache.cache.set("2", &req).await?;

    let res = sender.send_request(req).await?;

    println!("Response: {:?}", res.status());
    println!("Method: {:?}", method);
    println!("Host: {:?}", host);

    // Send the response back to the client
    Ok(res)
}

// curl --proxy localhost:6442 http://localhost:3000/spells/1 | jq
