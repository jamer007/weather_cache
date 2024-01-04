use axum::{
    extract::{Query, State},
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use redis_pool::{RedisPool, SingleRedisPool};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    // build our application with a route
    // let redis_client = redis::Client::open("redis://127.0.0.1/#insecure").unwrap();
    let redis_client = redis::Client::open("redis://redis/#insecure").expect("Error while testing the connection");
    let redis_pool = RedisPool::from(redis_client);
    println!("Redis connected");

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/accuweather", get(get_accuweather_forecast))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .with_state(redis_pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3003").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn get_accuweather_forecast(State(redis_pool): State<SingleRedisPool>, Query(coord): Query<Coord>) -> (StatusCode, Json<Vec<Forecast>>) {
    let mut connection = redis_pool.aquire().await.unwrap();
    let res: Option<String> = redis::cmd("GET")
        .arg(0)
        .query_async(&mut connection)
        .await
        .unwrap();

    return if res.is_none() {
        // fetch weather data
        let forecast = get_data_from_provider().await;
        let serialized_f = serde_json::to_string(&forecast).unwrap();

        println!("does not exist");
        let _: () = redis::pipe()
            .set(0, serialized_f)
            .ignore()
            .query_async(&mut connection)
            .await
            .unwrap();
        let _: () = redis::pipe()
            .expire(0, 3)
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

async fn get_data_from_provider() -> Vec<Forecast>{
    let forecast = vec![Forecast {
        Temperature: Temperature {
            Metric: Temp {
                Value: -10.0
            },
            Imperial: Temp {
                Value: -10.0
            },
        },
        RealFeelTemperature: RealFeelTemperature {
            Metric: Temp {
                Value: -10.0
            },
            Imperial: Temp {
                Value: 0.4
            },
        },
    }];
    forecast
}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[derive(Serialize, Deserialize)]
pub struct Temp {
    Value: f64
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

#[derive(Deserialize)]
struct Coord {
    lat: String,
    lon: String,
}