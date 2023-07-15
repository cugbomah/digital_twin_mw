use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension, Json,
};
use sea_orm::DatabaseConnection;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    database::core_user,
    helpers::policy_mgmt_helpers::check_policy,
    queries::{
        policy_queries,
        shared_data_queries::{self, SharedData},
        twin_queries,
    },
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

    // dbg!(&twin.enable_data_sharing);

    //Check redis for policy
    let mut store_key_model = "Policy:Models:".to_string();
    store_key_model = store_key_model + &twin.policy_id.unwrap().to_string();
    store_key_model = store_key_model + ":Verb:" + &endpoint_id.clone();

    let http_method = get_token_from_redis(redis_url.clone(), store_key_model.clone()).await?;

    *req.method_mut() = Method::from_bytes(http_method.as_bytes()).unwrap(); //Need to change this to match the request method; source would be core_policy_action.end_point_verb
                                                                             // dbg!(&req.body());
    let input_body_bytes = hyper::body::to_bytes(req.body_mut()).await.unwrap();
    *req.body_mut() = input_body_bytes.clone().into();

    let input_body = String::from_utf8(input_body_bytes.to_vec()).unwrap();
    // let input_body = serde_json::from_slice(&input_body_bytes).unwrap();

    // dbg!(&input_body);
    // if let Value::Object(obj) = input_body {
    //     process_object(obj);
    // } else {
    //     println!("Invalid JSON object");
    // }

    let return_result = client.request(req).await.unwrap();
    // dbg!(&return_result.body());
    let return_body_bytes = hyper::body::to_bytes(return_result.into_body())
        .await
        .unwrap();
    let return_result = Response::new(hyper::Body::from(return_body_bytes.clone()));
    let return_body = String::from_utf8(return_body_bytes.to_vec()).unwrap();

    // dbg!(&return_body);
    if twin.enable_data_sharing {
        let _store_result = shared_data_queries::store_usage_data(
            db.clone(),
            SharedData {
                model_id: Some(twin.model_id),
                input_data: Some(input_body),
                output_response: Some(return_body),
            },
        )
        .await?;
    }

    Ok(return_result)
}

// fn process_object(obj: serde_json::Map<String, Value>) {
//     for (key, value) in obj {
//         let formatted_value = format_value(&value);
//         println!("{}: {}", key, formatted_value);
//     }
// }

// fn format_value(value: &Value) -> String {
//     match value {
//         Value::String(s) => s.to_string(),
//         Value::Number(n) => {
//             if n.is_i64() {
//                 format!("{}", n.as_i64().unwrap())
//             } else if n.is_u64() {
//                 format!("{}", n.as_u64().unwrap())
//             } else {
//                 format!("{}", n.as_f64().unwrap())
//             }
//         }
//         _ => format!("{:?}", value),
//     }
// }
