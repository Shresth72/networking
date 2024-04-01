// #![allow(unused)]
use crate::{connection, state::StateInternal, ErrorType};

use hyper::{
    body::Incoming as Body, client::conn::http1 as Client, server::conn::http1 as Server,
    service::service_fn, Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{
    io::{self, Write},
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tower::ready_cache::cache;

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

    // Processing the request
    let req = Request::builder()
        .method(req.method())
        .uri(path)
        .header(hyper::header::HOST, authority)
        .body(req.into_body())
        .expect("Failed to build request");

    let res = sender.send_request(req).await?;

    // Without Parallel Execution, Cache should not be used
    // tokio::spawn(async move {
        // let cache = connection::conn().await.unwrap();
    
        // cache.cache.set("key1", res.body()).await.unwrap();
    // });

    println!("Response: {:?}", res.status());
    println!("Method: {:?}", method);
    println!("Host: {:?}", host);
    println!("Response Body: {:?}", res.body());

    // Send the response back to the client
    Ok(res)
}

// curl --proxy localhost:6442 http://localhost:3000/spells/1 | jq
