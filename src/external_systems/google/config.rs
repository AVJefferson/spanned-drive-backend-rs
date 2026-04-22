use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AccessTokenPayload {
    pub access_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GenerateTokenPayload {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
}