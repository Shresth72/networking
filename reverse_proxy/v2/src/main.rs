#![allow(unused_imports)]

mod connection;
mod state;

mod handler;
use dotenv::dotenv;
use handler::handle;

use fred::prelude::*;
use hyper::{
    body::Incoming as Body, client::conn::http1 as Client, server::conn::http1 as Server,
    service::service_fn, Request, Response,
};
use hyper_util::rt::TokioIo;
use std::{
    fmt::Debug,
    io::{self, Write},
    net::SocketAddr,
};
use tokio::net::{TcpListener, TcpStream};

pub type ErrorType = dyn std::error::Error + Send + Sync;

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

#[tokio::main]
async fn main() -> Result<(), Box<ErrorType>> {
    dotenv().ok();

    // Proxy Server

    let addr = SocketAddr::from(([127, 0, 0, 1], 6442));
    let listener = TcpListener::bind(addr).await?;
    println!("Listening on: {} on v2", addr);

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

// TODO 1: Caching
// TODO 2: CLI for processing config and send req to proxy
// TODO 3: Load Balancer
