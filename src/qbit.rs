use reqwest::RequestBuilder;
use serde::{Deserialize, Serialize};
use crate::error::Error;

#[derive(Debug, Serialize)]
struct LoginBody {
    username: String,
    password: String,
}

pub async fn login(request: RequestBuilder, username: &str, password: &str) -> Result<(), Error> {
    let body = LoginBody {
        username: username.to_string(),
        password: password.to_string(),
    };
    
    let resp = match request.form(&body).send().await {
        Ok(resp) => resp,
        Err(e) => return Err(Error::BadGateway(e)),
    };

    if !resp.status().is_success() {
        return Err(Error::InvalidCredentials);
    }
    
    Ok(())
}

pub async fn logout(request: RequestBuilder) -> Result<(), Error> {
    match request.send().await {
        Ok(_) => {}
        Err(e) => return Err(Error::BadGateway(e)),
    };
    
    Ok(())
}

#[derive(Debug, Deserialize)]
struct FilenameResponse {
    name: String,
}

pub async fn get_filename(request: RequestBuilder) -> Result<String, Error> {
    let resp = match request.send().await { 
        Ok(res) => res,
        Err(e) => return Err(Error::BadGateway(e)),
    };
    
    if !resp.status().is_success() {
        return Err(Error::BadRequest(resp.status().to_string()));
    }

    let filename = match resp
        .json::<Vec<FilenameResponse>>()
        .await {
        Ok(res) => res,
        Err(e) => { 
            println!("{:?}", e);
            return Err(Error::InvalidResponse(e)) 
        },
    };
    
    Ok(filename.first().unwrap().name.clone())
}

#[derive(Debug, Serialize)]
struct RenameBody {
    hash: String,
    #[serde(rename = "oldPath")]
    old_path: String,
    #[serde(rename = "newPath")]
    new_path: String,
}

pub async fn rename(request: RequestBuilder, hash: String, old_path: String, new_path: String) -> Result<(), Error> {
    let body = RenameBody {
        hash, old_path, new_path,
    };
    
    let resp = match request.form(&body).send().await {
        Ok(resp) => resp,
        Err(e) => return Err(Error::BadGateway(e)),
    };

    if !resp.status().is_success() {
        return Err(Error::BadRequest(resp.status().to_string()));
    }
    
    Ok(())
}
