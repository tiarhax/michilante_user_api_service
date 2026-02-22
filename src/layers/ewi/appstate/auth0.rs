use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::DecodingKey;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::layers::ewi::middleware::auth0::Claims;

#[derive(Clone)]
pub struct Auth0Config {
    pub domain: String,
    pub audience: String,
    pub issuer: String,
}

#[derive(Clone)]
pub struct Auth0State {
    pub auth0_config: Auth0Config,
    pub jwks_cache: Arc<RwLock<HashMap<String, DecodingKey>>>,
    pub http_client: Client,
}
#[derive(Debug, Deserialize)]
pub struct Jwk {
    pub kty: String,
    pub kid: String,
    pub r#use: Option<String>,
    pub n: String,
    pub e: String,
    pub alg: Option<String>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub roles: Vec<String>,
}

impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(User {
            id: claims.sub.clone(),
            roles: claims.roles.clone().unwrap_or_default(),
        })
    }
}


#[derive(Debug, Deserialize)]
pub struct Jwks {
    pub keys: Vec<Jwk>,
}

impl Auth0State {
    pub fn new(auth0_config: Auth0Config) -> Self {
        Self {
            auth0_config,
            jwks_cache: Arc::new(RwLock::new(HashMap::new())),
            http_client: Client::new(),
        }
    }

    pub async fn fetch_jwks(&self) -> Result<Jwks, Box<dyn std::error::Error + Send + Sync>> {
        let jwks_url = format!("https://{}/.well-known/jwks.json", self.auth0_config.domain);
        let response = self.http_client.get(&jwks_url).send().await?;
        let jwks: Jwks = response.json().await?;
        Ok(jwks)
    }

    pub async fn get_decoding_key(&self, kid: &str) -> Result<DecodingKey, Box<dyn std::error::Error + Send + Sync>> {
        {
            let cache = self.jwks_cache.read().await;
            if let Some(key) = cache.get(kid) {
                return Ok(key.clone());
            }
        }

        let jwks = self.fetch_jwks().await?;
        let mut cache = self.jwks_cache.write().await;

        for jwk in jwks.keys {
            if jwk.kid == kid {
                let decoding_key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;
                cache.insert(kid.to_string(), decoding_key.clone());
                return Ok(decoding_key);
            }
        }

        Err("Key not found in JWKS".into())
    }
}