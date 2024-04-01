#![allow(unused)]

use fred::interfaces::KeysInterface;
use fred::{clients::RedisPool, prelude::*};
use hyper::http::Version;
use hyper::HeaderMap;
use hyper::{body::Incoming, Request, Response};
use serde::Serialize;
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::ErrorType;

pub type CacheState = std::sync::Arc<StateInternal>;
pub struct StateInternal {
    pub cache: Cache,
}

impl StateInternal {
    pub fn new(redis: RedisPool) -> Self {
        StateInternal {
            cache: Cache { internal: redis },
        }
    }
}

pub struct Cache {
    internal: RedisPool,
}

// enum Request { method: GET, uri: /api, version: HTTP/1.1, headers: {"host": "localhost:8000"}, body: Body(Empty) }

#[derive(Debug)]
pub struct MyRequest<'a> {
    method: String,
    uri: String,
    version: Version,
    headers: &'a HeaderMap,
    body: &'a Incoming,
}

impl Cache {
    fn key_for_id(id: i64) -> String {
        format!("spell:{}", id)
    }

    pub async fn get(&self, key: &str) -> Result<Request<Incoming>, Box<ErrorType>> {
        if !self.internal.is_connected() {
            return Err("Redis connection is not established".into());
        }

        todo!()
    }

    pub async fn set(&self, key: &str, req: &Incoming) -> Result<(), Box<ErrorType>> {
        if !self.internal.is_connected() {
            return Err("Redis connection is not established".into());
        }

        // let method = req.method().to_string();
        // let uri = req.uri().to_string();
        // let version = req.version();
        // let headers = req.headers();
        // let body = req.body();

        // let request = MyRequest {
        //     method,
        //     uri,
        //     version,
        //     headers,
        //     body,
        // };

        // let value = format!("{:?}", request);
        // self.internal
        //     .set(
        //         key,
        //         value,
        //         Some(Expiration::EX(1)),
        //         Some(SetOptions::XX),
        //         false,
        //     )
        //     .await?;
        Ok(())
    }
}
