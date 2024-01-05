mod accuweather_service;

use std::env;
use axum::{
    extract::{Query, State},
    routing::{get},
    http::StatusCode,
    Json, Router,
};
use redis_pool::{RedisPool, SingleRedisPool};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // build our application with a route
    let redis_host = env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let redis_host_str = redis_host.as_str();
    let redis_client = redis::Client::open(format!("redis://{redis_host_str}/#insecure")).expect("Error while testing the connection");
    let redis_pool = RedisPool::from(redis_client);
    println!("Redis connected");

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/accuweather", get(get_accuweather_forecast))
        .with_state(redis_pool);

    // run our app with hyper, listening globally on port 3000
    let server_port = env::var("SERVER_PORT").unwrap_or("3003".to_string());
    let server_port_str = server_port.as_str();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{server_port_str}")).await.unwrap();
    println!("Server listening on port: {server_port}");
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_accuweather_forecast(State(redis_pool): State<SingleRedisPool>, Query(location_id): Query<LocationId>) -> (StatusCode, Json<Vec<Forecast>>) {
    let mut connection = redis_pool.aquire().await.unwrap();
    let location_id_key = location_id.location_id;
    let redis_key = format!("accuweather:{location_id_key}");
    let res: Option<String> = redis::cmd("GET")
        .arg(redis_key.clone())
        .query_async(&mut connection)
        .await
        .unwrap();

    return if res.is_none() {
        // fetch weather data
        let forecast = get_data_from_provider(location_id_key).await;
        let serialized_f = serde_json::to_string(&forecast).unwrap();

        println!("does not exist");
        let _: () = redis::pipe()
            .set(redis_key.clone(), serialized_f)
            .ignore()
            .query_async(&mut connection)
            .await
            .unwrap();
        let redis_cache_ttl = env::var("REDIS_CACHE_TTL_SEC").unwrap_or("600".to_string());
        let redis_cache_int = redis_cache_ttl.parse().unwrap();
        let _: () = redis::pipe()
            .expire(redis_key.clone(), redis_cache_int)
            .ignore()
            .query_async(&mut connection)
            .await
            .unwrap();

        (StatusCode::OK, Json(forecast))
    } else {
        let res_str = res.as_deref().unwrap_or("");
        let forecast: Vec<Forecast> = serde_json::from_str(res_str).unwrap();
        println!("exist");
        (StatusCode::OK, Json(forecast))
    }
}

async fn get_data_from_provider(location_id: String) -> Vec<Forecast>{
    let forecast_temps = accuweather_service::get_forecast_temps(location_id.clone()).await;
    let mut forecast = vec![Forecast {
        Temperature: Temperature {
            Metric: Temp {
                Value: 0.0
            },
            Imperial: Temp {
                Value: 0.0
            },
        },
        RealFeelTemperature: RealFeelTemperature {
            Metric: Temp {
                Value: 0.0
            },
            Imperial: Temp {
                Value: 0.0
            },
        },
    }];
    if forecast_temps.len() == 2 {
        forecast[0].Temperature.Metric.Value = forecast_temps[0];
        forecast[0].RealFeelTemperature.Metric.Value = forecast_temps[1];
    }

    forecast
}

#[derive(Serialize, Deserialize)]
pub struct Temp {
    Value: f32
}
#[derive(Serialize, Deserialize)]
pub struct RealFeelTemperature {
    Metric: Temp,
    Imperial: Temp
}
#[derive(Serialize, Deserialize)]
pub struct Temperature {
    Metric: Temp,
    Imperial: Temp
}
#[derive(Serialize, Deserialize)]
pub struct Forecast {
    Temperature: Temperature,
    RealFeelTemperature: RealFeelTemperature
}
#[derive(Serialize, Deserialize)]
struct LocationId {
    location_id: String
}