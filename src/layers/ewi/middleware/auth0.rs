use axum::{extract::{Request, State}, http::HeaderMap, middleware::Next, response::Response};
use jsonwebtoken::{decode, decode_header, Algorithm, Validation};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::layers::ewi::appstate::{auth0::Auth0State, AppState};



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub aud: String,
    pub iss: String,
    pub exp: usize,
    pub iat: usize,
    pub scope: Option<String>,
    pub permissions: Option<Vec<String>>,
}


async fn validate_jwt(
    state: &Auth0State,
    token: &str,
) -> Result<Claims, Box<dyn std::error::Error + Send + Sync>> {
    // Decode header to get kid
    let header = decode_header(token)?;
    let kid = header.kid.ok_or("Missing kid in JWT header")?;

    // Get decoding key
    let decoding_key = state.get_decoding_key(&kid).await?;

    // Set up validation parameters
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&state.auth0_config.audience]);
    validation.set_issuer(&[&state.auth0_config.issuer]);

    // Decode and validate token
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;

    Ok(token_data.claims)
}

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let auth_header = headers.get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_str = auth_header.to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if !auth_str.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(auth_str[7..].to_string())
}

pub async fn auth0_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_bearer_token(&headers).map_err( |e| {
        tracing::error!("Eror extracting token");
        e
    })?;
    tracing::info!("received token: {}", token);
    let claims = validate_jwt(&state.auth0, &token).await
        .map_err(|e| {
            tracing::error!("error validating jwt {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    req.extensions_mut().insert(claims);

    Ok(next.run(req).await)
}