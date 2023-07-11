use axum::{
    middleware,
    routing::{delete, get, post, put},
    Extension, Router,
};

use crate::{
    app_state::AppState,
    middleware::require_authentication::require_authentication,
    routes::{
        models::{
            create_model::create_model,
            delete_model::delete_model,
            get_all_models::{get_all_owner_models, get_all_publsihed_models},
            publish_model::publish_model,
            unpublish_model::unpublish_model,
        },
        policys::{
            create_policy::create_policy,
            get_latest_policy::{get_all_model_policies, get_latest_model_policy},
        },
        twins::{
            delete_twin::soft_delete_twin,
            get_all_user_twins::get_all_user_twins,
            get_one_user_twin::get_one_user_twin,
            subscribe_to_model::subscribe,
            twin_operations::{start_twins, stop_twins},
            twin_usage::remote_request_handler,
        },
        users::{
            login::login,
            logout::logout,
            signup::{signup_owner, signup_user},
        },
    },
};
use hyper::Client;
use tower_cookies::CookieManagerLayer;

pub async fn create_router(app_state: AppState) -> Router {
    let client = Client::new();

    Router::new()
        .route(
            "/user/twins/:twin_id/action/:endpoint_id",
            post(remote_request_handler),
        )
        .layer(Extension(client))
        .route("/api/users/logout", post(logout))
        .route("/user/hello", get(|| async { "Hello, World!" }))
        .route("/owner/bye", get(|| async { "Goodbye, World!" }))
        .route("/owner/models", get(get_all_owner_models))
        .route("/user/models", get(get_all_publsihed_models))
        .route("/user/models/:model_id/subscribe", post(subscribe))
        .route("/user/twins", get(get_all_user_twins))
        .route("/user/twins/:twin_id", get(get_one_user_twin))
        .route("/user/twins/:twin_id", delete(soft_delete_twin))
        .route("/user/twins/:twin_id/start", put(start_twins))
        .route("/user/twins/:twin_id/stop", put(stop_twins))
        .route("/owner/deploy", post(create_model))
        .route("/owner/:model_id/publish", put(publish_model))
        .route("/owner/:model_id/unpublish", put(unpublish_model))
        .route("/owner/:model_id/policy", post(create_policy))
        .route("/owner/:model_id/policy", get(get_latest_model_policy))
        .route("/owner/:model_id/policies", get(get_all_model_policies))
        .route("/owner/:model_id", delete(delete_model))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            require_authentication,
        ))
        .route("/login", get(login))
        .route("/users/signup", post(signup_user))
        .route("/owners/signup", post(signup_owner))
        .route("/users/login", post(login))
        .layer(CookieManagerLayer::new())
        .with_state(app_state.clone())
}
