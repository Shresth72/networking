use crate::{state, ErrorType};

use dotenv::dotenv;
use fred::prelude::*;
use std::error::Error;
use std::sync::Arc;

pub async fn conn() -> Result<Arc<state::StateInternal>, Box<ErrorType>> {
    dotenv().ok();

    let redis_url = match std::env::var("REDIS_URL")?.as_str() {
        "" => "redis://localhost:5434".to_string(),
        x => x.to_string(),
    };

    let pool_size = 8;
    let config = RedisConfig::from_url(&redis_url)?;

    let redis_pool = Builder::from_config(config)
        .with_performance_config(|config| {
            config.auto_pipeline = true;
        })
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(pool_size)
        .expect("Failed to create redis pool");

    if std::env::var("REDIS_URL")? != "" {
        redis_pool.init().await.expect("Failed to connect to redis");
        // let _ = redis_pool.flushall::<i32>(false).await;
    }

    Ok(Arc::new(state::StateInternal::new(redis_pool)))
}

//
//
//
//
//
pub async fn _connect_fred() -> Result<RedisClient, Box<dyn Error>> {
    dotenv().ok();

    // Redis Cache
    let redis_url = match std::env::var("REDIS_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("No Redis URL found");
            "redis://localhost:3003".to_string()
        }
    };

    let config = RedisConfig::from_url(&redis_url)?;
    let client = Builder::from_config(config).build()?;
    let _connection_task = client.init().await?;

    client
        .set(
            "foo",
            "bar",
            Some(Expiration::EX(1)),
            Some(SetOptions::NX),
            false,
        )
        .await?;

    println!("Foo: {:?}", client.get::<Option<String>, _>("foo").await?);

    if client.is_connected() {
        println!("Connected to Redis");
    } else {
        println!("Failed to connect to Redis");
        std::process::exit(1);
    }

    Ok(client)
}
