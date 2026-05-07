use super::GoogleClient;

impl GoogleClient {
    pub async fn fetch_refresh_token(
        &self,
        code: String,
        code_verifier: String,
        redirect_uri: String,
    ) -> anyhow::Result<serde_json::Value> {
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("code", &code),
            ("code_verifier", &code_verifier),
            ("grant_type", &"authorization_code".to_string()),
            ("redirect_uri", &redirect_uri),
        ];

        let response = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }

    pub async fn fetch_access_token(
        &self,
        refresh_token: String,
    ) -> anyhow::Result<serde_json::Value> {
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("refresh_token", &refresh_token),
            ("grant_type", &"refresh_token".to_string()),
        ];

        let response = self
            .http_client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }

    pub async fn fetch_user_info(
        &self,
        access_token: String,
    ) -> anyhow::Result<serde_json::Value> {
        let response = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }
}
