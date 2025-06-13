mod error;
mod qbit;

use std::{env};
use std::net::SocketAddr;
use axum::{Router, Form};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tracing::error;
use tracing_subscriber::EnvFilter;
use regex::Regex;
use crate::error::Error;

const LOGIN_PATH: &str = "/api/v2/auth/login";
const LOGOUT_PATH: &str = "/api/v2/auth/logout";
const FILES_PATH: &str = "/api/v2/torrents/files";
const RENAME_PATH: &str = "/api/v2/torrents/renameFile";

#[derive(Clone)]
struct AppState {
    client: Client,
    qbit_url: String,
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(3000);
    
    let qbit_url = env::var("QBIT_URL")
        .ok()
        .unwrap_or("http://localhost:8080".to_string());
    
    let username = env::var("QBIT_USERNAME")
        .ok()
        .unwrap_or("".to_string());
    
    let password = env::var("QBIT_PASSWORD")
        .ok()
        .unwrap_or("".to_string());

    let client = Client::builder()
        .cookie_store(true)
        .build()
        .unwrap();
    
    let state = AppState {
        client, qbit_url, username, password,
    };

    let app = Router::new()
        .route("/rename", post(handler))
        .with_state(state);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await.unwrap();
}

#[derive(Deserialize, Debug)]
struct RenameTorrent {
    hash: String,
    tag: String,
}

async fn handler(State(state): State<AppState>, Form(payload): Form<RenameTorrent>) -> Response {
    let hash = payload.hash.clone();
    if payload.tag.is_empty() {
        return StatusCode::NO_CONTENT.into_response();
    }

    let use_login = !state.username.is_empty();
    if use_login { 
        let login_url = state.qbit_url.clone() + LOGIN_PATH;
        
        match qbit::login(
           state.client.post(&login_url),
           &state.username,
           &state.password,
        ).await {
            Ok(_) => {},
            Err(e) => { return e.into_response(); }
        };
    }

    let logout_and_return = |error_response: Error| {
        if use_login {
            let logout_url = format!("{}{}", &state.qbit_url, LOGOUT_PATH);
            let logout_request = state.client.post(&logout_url);
            tokio::spawn(async move { qbit::logout(logout_request).await });
        }
        error_response.into_response()
    };

    let files_url = format!("{}{}?hash={}", &state.qbit_url, FILES_PATH, hash);
    let files_request = state.client.get(&files_url);

    let filename = match qbit::get_filename(files_request).await {
        Ok(s) => s,
        Err(e) => return logout_and_return(e),
    };

    let new_filename = match process_filename(&filename, payload) {
        Ok(filename) => filename,
        Err(e) => {
            error!("Error processing torrent {}", hash);
            return logout_and_return(e)
        },
    };
    
    let rename_url = state.qbit_url.clone() + RENAME_PATH;
    let rename_request = state.client.post(&rename_url);
    match qbit::rename(rename_request, hash, filename, new_filename).await {
        Ok(_) => {}
        Err(e) => return logout_and_return(e),
    }

    let logout_url = format!("{}{}", &state.qbit_url, LOGOUT_PATH);
    let logout_request = state.client.post(&logout_url);
    tokio::spawn(async move { qbit::logout(logout_request).await });
    
    StatusCode::OK.into_response()
}

fn process_filename(filename: &str, payload: RenameTorrent) -> Result<String, Error> {
    let Some((pattern, offset)) = payload.tag.rsplit_once('@') else {
        return Err(Error::InvalidTag(payload.tag));
    };

    let Ok(regex) = Regex::new(pattern) else {
        return Err(Error::InvalidRegex(pattern.to_string()));
    };

    let Ok(offset) = offset.parse::<i32>() else {
        return Err(Error::InvalidOffset(offset.to_string()));
    };

    let Some(regex_match) = regex.captures(filename) else {
        return Err(Error::NoMatch(filename.to_string(), pattern.to_string()));
    };

    let Some(ep_number_group) = regex_match.get(1) else {
        return Err(Error::NoGroup(pattern.to_string()));
    };

    let ep_number = ep_number_group.as_str();
    let Ok(ep_number_int) = ep_number.parse::<i32>() else {
        return Err(Error::GroupNotANumber(ep_number.to_string()));
    };

    let new_filename = regex.replace(filename, |caps: &regex::Captures| {
        let old = caps.get(0).unwrap().as_str();
        old.replace(ep_number, &(ep_number_int + offset).to_string())
    });

    Ok(new_filename.to_string())
}
