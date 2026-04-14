use crate::{clients::Clients, constants, logger::Logger, routes, state::AppState};

use axum;
use dashmap::DashMap;
use std::{env, sync::Arc};

pub async fn start_server() {
    Logger::new(constants::LOG_CHANNEL_SIZE).expect("Failed to initialize logger");

    let clients = Clients::new_from_env_variables().await;
    let auth_tokens = DashMap::new();

    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "local".to_string());
    if environment == "local" || environment == "dev" {
        auth_tokens.insert("test".to_string(), vec!["token".to_string()]);
        auth_tokens.insert("admin".to_string(), vec!["admin".to_string()]);
    }

    if let Ok(entries) = std::fs::read_dir("allowed_clients") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "key" {
                        if let (Some(file_stem), Ok(content)) =
                            (path.file_stem(), std::fs::read_to_string(&path))
                        {
                            if let Ok(permissions) = serde_json::from_str::<Vec<String>>(&content) {
                                let key = file_stem.to_string_lossy().into_owned();
                                auth_tokens.insert(key.clone(), permissions.clone());
                                let display_key = if key.len() > 10 {
                                    format!("{}...{}", &key[..5], &key[key.len() - 5..])
                                } else {
                                    key
                                };
                                log::info!(
                                    "key: {} is added to auth_tokens with permissions {:?}",
                                    display_key,
                                    permissions
                                );
                            }
                        }
                    } else if extension == "blocked"
                        || extension == "deleted"
                        || extension == "disabled"
                    {
                        let key = path.file_name().unwrap().to_string_lossy().into_owned();
                        let display_key = if key.len() > 10 {
                            format!("{}...{}", &key[..5], &key[key.len() - 5..])
                        } else {
                            key
                        };
                        log::warn!("key: {} is {}", display_key, extension.to_string_lossy());
                    }
                }
            }
        }
    } else {
        log::warn!("'allowed_clients' directory not found. No client keys will be loaded.");
    }

    let app_state = Arc::new(AppState {
        clients,
        auth_tokens,
    });

    let port =
        env::var("SERVER_PORT").unwrap_or_else(|_| constants::DEFAULT_SERVER_PORT.to_string());
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| constants::DEFAULT_SERVER_HOST.to_string());
    let addr = format!("{}:{}", host, port);

    log::info!(
        "Starting server... at http://{}. TraceID: {}",
        addr,
        constants::APP_TRACE_ID.as_str()
    );

    let app = routes::routes(app_state);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server crashed unexpectedly");
}
