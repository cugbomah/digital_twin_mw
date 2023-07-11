use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension,
};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    database::core_user,
    helpers::policy_mgmt_helpers::check_policy,
    queries::{policy_queries, twin_queries},
    utilities::{
        app_error::AppError,
        redis_connection_wrapper::RedisConnWrapper,
        redis_helper::{get_token_from_redis, get_token_ttl, store_token_in_redis},
    },
};
use hyper::{Body, Client, Method, Request as HyperRequest, StatusCode, Uri};

pub async fn remote_request_handler(
    State(db): State<DatabaseConnection>,
    State(redis_url): State<RedisConnWrapper>,
    Extension(client): Extension<Client<hyper::client::HttpConnector>>,
    Extension(user): Extension<core_user::Model>,
    Path(path_params): Path<(Uuid, String)>,
    // Path(endpoint_id): Path<String>,
    mut req: HyperRequest<Body>,
) -> Result<Response<Body>, AppError> {
    let (twin_id, endpoint_id) = path_params;

    //Get twin from db by using appropriate twin_queries
    let (twin, _twin_status) = twin_queries::get_one_user_twin(&db, twin_id, user.id).await?;

    //Check if twin is running
    if twin.twin_status_id != 2 {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "Twin is not running",
        ));
    }

    //Check if twin.policy_id is not null
    if !twin.policy_id.is_none() {
        //Call async fn check_policy to check if user has access to endpoint
        let _policy_check = check_policy(
            &db,
            twin_id.clone(),
            endpoint_id.clone(),
            user.id.clone(),
            twin.policy_id.clone().unwrap(),
            redis_url.clone(),
        )
        .await?;
    }

    // Construct URI
    let uri = format!(
        "http://localhost:{}/{}",
        twin.twin_port.unwrap(),
        endpoint_id
    );

    // Prepare request and return
    *req.uri_mut() = Uri::try_from(uri).unwrap();

    //Check redis for policy
    let mut store_key_model = "Policy:Models:".to_string();
    store_key_model = store_key_model + &twin.policy_id.unwrap().to_string();
    store_key_model = store_key_model + ":Verb:" + &endpoint_id.clone();

    let http_method = get_token_from_redis(redis_url.clone(), store_key_model.clone()).await?;

    *req.method_mut() = Method::from_bytes(http_method.as_bytes()).unwrap(); //Need to change this to match the request method; source would be core_policy_action.end_point_verb

    let return_result = client.request(req).await.unwrap();
    dbg!(&return_result);

    Ok(return_result)
}
