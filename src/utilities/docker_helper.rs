use axum::http::StatusCode;
use axum::Json;
use bollard::container::Config;
use bollard::container::CreateContainerOptions;
use bollard::container::{RemoveContainerOptions, StartContainerOptions, StopContainerOptions};
use bollard::image::CreateImageOptions;
use bollard::service::HostConfig;
use bollard::Docker;
use futures_util::TryStreamExt;
use maplit::hashmap;
// use std::collections::HashMap;
// use std::convert::Infallible;
use bollard::models::EndpointSettings;
use bollard::network::{ConnectNetworkOptions, CreateNetworkOptions};
use sea_orm::DatabaseConnection;
use sea_orm::IntoActiveModel;
use sea_orm::Set;
use std::default::Default;

use crate::database::core_model_component;
use crate::database::core_twin;
use crate::database::core_twin_component;
use crate::queries::twin_queries;

use super::app_error::AppError;
use super::redis_connection_wrapper::RedisConnWrapper;
use super::redis_helper::delete_token_from_redis;
use super::redis_helper::get_token_from_redis;
use super::redis_helper::store_token_in_redis;
use rand::Rng;

// const IMAGE: &str = "sandiek/digitalfarm:latest";

//Create a function for creating a docker network
async fn create_docker_network(network_name: String) -> Result<(), AppError> {
    // Create a Docker client
    let docker = Docker::connect_with_local_defaults().map_err(|error| {
        eprintln!("Error connecting to docker daemon: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Connection error")
    })?;

    //Create a docker network
    let network_name = network_name;
    let network_config = CreateNetworkOptions {
        name: network_name,
        ..Default::default()
    };
    docker
        .create_network(network_config)
        .await
        .map_err(|error| {
            eprintln!("Error creating network: {:?}", error);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error creating network")
        })?;

    // dbg!(network.id);
    Ok(())
}

async fn create_container(
    network_name: String,
    image_name: String,
    container_name: &String,
    component_alias: String,
    container_port: i32,
    host_port: i32,
) -> Result<String, AppError> {
    // Create a Docker client
    let docker = Docker::connect_with_local_defaults().map_err(|error| {
        eprintln!("Error connecting to docker daemon: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Connection error")
    })?;

    // Specify the image you want to run
    let image_name = image_name;
    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: image_name.clone(),
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await
        .map_err(|error| {
            eprintln!("Error creating docker image: {:?}", error);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error creating image")
        })?;

    let container_port = container_port.to_string() + "/tcp";
    //container_port = &container_port.push("/tcp");
    let host_port = host_port.to_string();

    let options = Some(CreateContainerOptions {
        name: container_name.as_str(),
        platform: None,
    });

    //assign port_bindings to None if host_port is 0
    let port_bindings = if host_port != "0" {
        hashmap! {
                container_port.to_owned()=>Some(vec![bollard::service::PortBinding{
                host_ip:Some("0.0.0.0".to_owned()),
                host_port: Some(host_port.to_string())
            }])
        }
    } else {
        let port_bindings: std::collections::HashMap<
            String,
            Option<Vec<bollard::service::PortBinding>>,
        > = std::collections::HashMap::new();
        port_bindings
    };

    let host_config = HostConfig {
        port_bindings: Some(port_bindings),
        ..Default::default()
    };

    let alpine_config = Config {
        image: Some(image_name.as_str()),
        host_config: Some(host_config),
        tty: Some(true),
        ..Default::default()
    };

    // Create the container
    let id = docker
        .create_container::<&str, &str>(options, alpine_config)
        .await
        .map_err(|error| {
            eprintln!("Error creating container: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error creating container",
            )
        })?
        .id;

    // //add a network alias to the container and attach the container to the network
    // let id = id.clone();
    // let config = CreateContainerOptions {
    //     name: container_name.as_str(),
    //     platform: None,
    // };

    let config = ConnectNetworkOptions {
        container: &id,
        endpoint_config: EndpointSettings {
            aliases: Some(vec![component_alias]),
            ..Default::default()
        },
    };
    let network_id = &network_name;
    docker
        .connect_network(network_id, config)
        .await
        .map_err(|error| {
            eprintln!("Error connecting container to network: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error connecting container to network",
            )
        })?;

    // Start the container
    docker
        .start_container::<String>(&id, None)
        .await
        .map_err(|error| {
            eprintln!("Error starting container: {:?}", error);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Connection error")
        })?;

    let my_msg = docker.ping().await.map_err(|error| {
        eprintln!("Error pinging container: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error pinging container")
    })?;

    Ok(my_msg)
}

async fn stop_container(container_name: &String) -> Result<(), AppError> {
    // Create a Docker client
    let docker = Docker::connect_with_local_defaults().map_err(|error| {
        eprintln!("Error connecting to docker daemon: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Connection error")
    })?;
    let options = Some(StopContainerOptions { t: 30 });

    docker
        .stop_container(container_name, options)
        .await
        .map_err(|error| {
            eprintln!("Error stopping docker container: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error stopping container",
            )
        })?;

    Ok(())
}

async fn start_container(container_name: &String) -> Result<(), AppError> {
    // Create a Docker client
    let docker = Docker::connect_with_local_defaults().map_err(|error| {
        eprintln!("Error connecting to docker daemon: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Connection error")
    })?;

    docker
        .start_container(container_name, None::<StartContainerOptions<String>>)
        .await
        .map_err(|error| {
            eprintln!("Error starting docker container: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error starting container",
            )
        })?;

    Ok(())
}

async fn remove_container(container_name: &String) -> Result<(), AppError> {
    // Create a Docker client
    let docker = Docker::connect_with_local_defaults().map_err(|error| {
        eprintln!("Error connecting to docker daemon: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Connection error")
    })?;

    let options = Some(RemoveContainerOptions {
        force: true,
        ..Default::default()
    });

    docker
        .remove_container(container_name, options)
        .await
        .map_err(|error| {
            eprintln!("Error removing docker container: {:?}", error);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error removing container",
            )
        })?;

    Ok(())
}

//Create a function for removing a docker network
async fn remove_docker_network(network_name: &String) -> Result<Json<String>, AppError> {
    // Create a Docker client
    let docker = Docker::connect_with_local_defaults().map_err(|error| {
        eprintln!("Error connecting to docker daemon: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Connection error")
    })?;

    docker.remove_network(network_name).await.map_err(|error| {
        eprintln!("Error removing network: {:?}", error);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error removing network")
    })?;

    Ok(Json("Network removed successfully!".to_string()))
}

pub async fn create_docker_model(
    db: &DatabaseConnection,
    model: &core_twin::Model,
    model_components: Vec<core_twin_component::Model>,
    redis_url: RedisConnWrapper,
    username: &String,
) -> Result<Json<String>, AppError> {
    let mut twin_port: i32 = 0;

    //obtain network name from model.name by replacing whitespaces with underscores
    // let network_number = rand::thread_rng().gen::<i32>();
    let network_number = rand::thread_rng().gen_range(0..9999);
    let mut network_name = model.name.replace(" ", "_");

    let formatted_username = username.replace("@", "");
    let formatted_username = formatted_username.replace(".", "");
    let formatted_username = formatted_username.as_str();

    network_name.push_str("_network_");
    network_name.push_str(formatted_username);
    network_name.push_str("_");
    network_name.push_str(&network_number.to_string());
    network_name = network_name.to_lowercase();

    create_docker_network(network_name.clone()).await?;

    //Iterate through model_components and create docker containers using create_container()
    for comp in model_components.clone() {
        let twin_component = comp.clone();
        let mut container_name = comp.name.replace(" ", "_");
        container_name.push_str("_container_");
        container_name.push_str(formatted_username);
        container_name.push_str("_");
        container_name.push_str(&network_number.to_string());
        container_name = container_name.to_lowercase();

        //Check if model is exposed
        //If exposed, generate a random number between 8000 and 9999, and assign it to host_port
        //If not exposed, assign host_port to 0
        let host_port = if comp.is_exposed {
            //recursively generate a random number between 8000 and 9999, and check if it is already in use
            //if it is in use, generate another random number
            //if it is not in use, assign it to host_port
            let mut host_port: i32;
            loop {
                host_port = rand::thread_rng().gen_range(8000..9999);
                let mut store_key = "Ports:host:".to_string();
                store_key = store_key + &host_port.to_string();
                // store_key = store_key + formatted_username + ":" + &container_name;
                let redis_token = get_token_from_redis(redis_url.clone(), store_key.clone())
                    .await
                    .unwrap_or_default();
                if redis_token == "" {
                    break;
                }
            }

            host_port
        } else {
            0
        };

        create_container(
            network_name.clone(),
            comp.image_source,
            &container_name,
            comp.component_alias.clone().unwrap_or_default(),
            comp.container_port.clone().unwrap_or_default(),
            host_port,
        )
        .await?;
        //store host_port in redis if host_port is not 0
        if host_port != 0 {
            let mut store_key = "Ports:host:".to_string();
            store_key = store_key + &host_port.to_string();
            // store_key = store_key + formatted_username + ":" + &container_name;
            store_token_in_redis(
                redis_url.clone(),
                store_key.clone(),
                host_port.to_string(),
                usize::MAX,
            )
            .await?;

            //Update component's host_port and container_name
            let mut comp = twin_component.into_active_model();
            comp.host_port = Set(Some(host_port.clone()));
            comp.container_name = Set(Some(container_name));
            twin_queries::save_active_coretwin_component(&db, comp).await?;

            twin_port = host_port;
        } else {
            //Update component's container_name
            let mut comp = twin_component.into_active_model();
            comp.container_name = Set(Some(container_name));
            twin_queries::save_active_coretwin_component(&db, comp).await?;
        }
    }

    //Update model's network_name
    let mut model = model.clone().into_active_model();
    model.twin_status_id = Set(2); //Set twin status to "Started"
    model.network_name = Set(Some(network_name.clone()));

    //Check if twin_Port is not None, then set to model.twin_Port
    if twin_port != 0 {
        model.twin_port = Set(Some(twin_port));
    }

    twin_queries::save_active_coretwin(&db, model).await?;

    Ok(Json("Containers created successfully!".to_string()))
}

pub async fn start_docker_model(
    model_components: Vec<core_twin_component::Model>,
    _username: &String,
) -> Result<Json<String>, AppError> {
    //Iterate through model_components and create docker containers using create_container()
    for comp in model_components.clone() {
        // let mut container_name = comp.name.replace(" ", "_");
        // // container_name.push_str("_container_");
        // // container_name.push_str(formatted_username);
        // container_name = container_name.to_lowercase();

        start_container(&comp.container_name.unwrap()).await?;
    }

    Ok(Json("Containers started successfully!".to_string()))
}

pub async fn stop_docker_model(
    model_components: Vec<core_twin_component::Model>,
    _username: &String,
) -> Result<Json<String>, AppError> {
    // //Stop and remove all containers in the network
    // let formatted_username = username.replace("@", "");
    // let formatted_username = formatted_username.replace(".", "");
    // let formatted_username = formatted_username.as_str();

    for comp in model_components {
        // let mut container_name = comp.name.replace(" ", "_");
        // container_name.push_str("_container_");
        // container_name.push_str(formatted_username);
        // container_name = container_name.to_lowercase();

        //Stop container
        stop_container(&comp.container_name.unwrap()).await?;
    }

    Ok(Json("Containers stopped successfully!".to_string()))
}

pub async fn remove_docker_model(
    model: &core_twin::Model,
    model_components: Vec<core_twin_component::Model>,
    redis_url: RedisConnWrapper,
) -> Result<Json<String>, AppError> {
    // //Stop and remove all containers in the network
    // let formatted_username = username.replace("@", "");
    // let formatted_username = formatted_username.replace(".", "");
    // let formatted_username = formatted_username.as_str();

    for comp in model_components {
        // let mut container_name = comp.name.replace(" ", "_");
        // container_name.push_str("_container_");
        // container_name.push_str(formatted_username);
        // container_name = container_name.to_lowercase();

        //Stop container
        remove_container(&comp.container_name.unwrap()).await?;
        //Delete redis key
        let mut store_key = "Ports:host:".to_string();
        store_key = store_key + comp.host_port.unwrap_or_default().to_string().as_str();
        delete_token_from_redis(redis_url.clone(), store_key).await?;
    }

    //obtain network name from model.name by replacing whitespaces with underscores
    // let mut network_name = model.name.replace(" ", "_");

    // network_name.push_str("_network_");
    // network_name.push_str(formatted_username);
    // network_name = network_name.to_lowercase();

    let _msg = remove_docker_network(&model.clone().network_name.unwrap()).await?;
    Ok(Json("Containers removed successfully!".to_string()))
}
