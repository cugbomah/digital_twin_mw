use dotenvy::dotenv;
use dotenvy_macro::dotenv;

use digital_twin_mw::{
    app_state::AppState,
    run,
    utilities::token_wrapper::TokenWrapper,
    utilities::{
        redis_connection_wrapper::RedisConnWrapper, token_duration_wrapper::TokenDurationWrapper,
    },
};
use sea_orm::Database;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = dotenv!("DATABASE_URL");
    let jwt_secret = dotenv!("JWT_SECRET").to_owned();
    let token_duration = dotenv!("TOKEN_DURATION")
        .to_string()
        .parse::<i64>()
        .unwrap();
    let redis_url = dotenv!("REDIS_URL");
    let db = match Database::connect(database_url).await {
        Ok(db) => db,
        Err(error) => {
            eprintln!("Error Connecting to the Database {:?}", error);
            panic!();
        }
    };

    //Store APP_STATE
    let app_state = AppState {
        db,
        jwt_secret: TokenWrapper(jwt_secret),
        token_duration: TokenDurationWrapper(token_duration),
        redis_url: RedisConnWrapper(redis_url.to_string()),
    };

    run(app_state).await;

    // endregion: Start Server
}

// // region: Handler
// async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
//     println!("->> {:<12}  - handler_hello - {params:?}", "HANDLER");

//     //let name = params.name.as_ref().map(|name| &**name).unwrap_or("World");
//     let name = params.name.as_deref().unwrap_or("World");
//     Html(format!("Hello, <strong>{name}!</strong>"))
// }
// // endregion: Handler

// // region: Handler2
// async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
//     println!("->> {:<12}  - handler_hello2", "HANDLER");
//     Html(format!("Hello, <strong>{name}!</strong>"))
// }
// // endregion: Handler2

// #[derive(Deserialize, Debug)]
// struct HelloParams {
//     name: Option<String>,
// }
