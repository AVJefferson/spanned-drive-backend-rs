pub mod config;

use self::config::GoogleConfig;

pub struct GoogleClient {
    pub config: GoogleConfig,
}

impl GoogleClient {
    pub async fn new(config: GoogleConfig) -> anyhow::Result<Self> {
        Ok(Self { config })
    }

    pub async fn fetch_refresh_token(
        &self,
        code: String,
        code_verifier: String,
        redirect_uri: String,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("code", &code),
            ("code_verifier", &code_verifier),
            ("grant_type", &"authorization_code".to_string()),
            ("redirect_uri", &redirect_uri),
        ];

        let response: reqwest::Response = client
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
        let client = reqwest::Client::new();
        let params = [
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("refresh_token", &refresh_token),
            ("grant_type", &"refresh_token".to_string()),
        ];

        let response: reqwest::Response = client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }

    pub async fn fetch_user_info(&self, access_token: String) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let response: reqwest::Response = client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }

    pub async fn fetch_drive_info(
        &self,
        access_token: String,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let response: reqwest::Response = client
            .get("https://www.googleapis.com/drive/v3/about?fields=storageQuota")
            .bearer_auth(access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        Ok(json)
    }

    pub async fn list_appdata_files(
        &self,
        access_token: String,
        query: String,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let response: reqwest::Response = client
            .get(format!(
                "https://www.googleapis.com/drive/v3/files?q={query}&spaces=appDataFolder"
            ))
            .bearer_auth(access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;

        let json = json
            .get("files")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));
        Ok(json)
    }

    pub async fn get_primary_file_id(&self, access_token: String) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let response: reqwest::Response = client
            .get("https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='is_primary.json'")
            .bearer_auth(access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        if let Some(files) = json.get("files").and_then(|f| f.as_array()) {
            if !files.is_empty() {
                let file_id = files[0].get("id").and_then(|id| id.as_str()).unwrap_or("");

                return Ok(file_id.to_string());
            }
        }

        Ok("".to_string())
    }

    pub async fn set_is_primary(&self, access_token: String) -> anyhow::Result<()> {
        let client = reqwest::Client::new();

        let response = client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(access_token)
            .header("Content-Type", "multipart/related; boundary=foo_bar_baz")
            .body(
                "--foo_bar_baz\r\n\
                 Content-Type: application/json; charset=UTF-8\r\n\r\n\
                 {\"name\": \"is_primary.json\", \"parents\": [\"appDataFolder\"]}
                 --foo_bar_baz\r\n\
                 Content-Type: application/json\r\n\r\n\
                 {\"is_primary\": true}\r\n\
                 --foo_bar_baz--",
            )
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to set is_primary: {}",
                response.text().await?
            ))
        }
    }

    pub async fn get_secondary_drives(&self, access_token: String) -> anyhow::Result<Vec<String>> {
        let client = reqwest::Client::new();

        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email == "" {
            return Ok(vec![]);
        }
        let primary_email = primary_email.replace("@", "__at__").replace(".", "__dot__");

        let response: reqwest::Response = client
            .get(format!("https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name contains 'sdrive---secondary-drive---{}---'", primary_email.clone()))
            // .get("https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name contains 'sdrive_secondary_drive_primary'")
            .bearer_auth(access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        let mut secondary_drives = Vec::new();

        if let Some(files) = json.get("files").and_then(|f| f.as_array()) {
            for file in files {
                if let Some(name) = file.get("name").and_then(|n| n.as_str()) {
                    if let Some(id) = file.get("id").and_then(|id| id.as_str()) {
                        secondary_drives.push(format!(
                            "{}{}",
                            id,
                            name.to_string()
                                .replace("sdrive---secondary-drive---", "")
                                .replace(&primary_email, "")
                                .replace("---", "||")
                                .replace("__at__", "@")
                                .replace("__dot__", ".")
                                .replace(".json", "")
                        ));
                    }
                }
            }
        }

        Ok(secondary_drives)
    }

    pub async fn set_secondary_drive(
        &self,
        access_token: String,
        drive_provider: String,
        new_secondary_drive_email: String,
    ) -> anyhow::Result<()> {
        let client = reqwest::Client::new();

        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email == "" {
            return Err(anyhow::anyhow!("Failed to fetch user info"));
        }
        let primary_email = primary_email.replace("@", "__at__").replace(".", "__dot__");

        let file_name = format!(
            "sdrive---secondary-drive---{}---{}---{}.json",
            primary_email,
            drive_provider,
            new_secondary_drive_email
                .replace("@", "__at__")
                .replace(".", "__dot__")
        );

        let existing_files_response = self
            .list_appdata_files(access_token.clone(), format!("name = '{}'", file_name))
            .await?;

        // if filename already exists in the array, return early
        if let Some(files) = existing_files_response.as_array() {
            if !files.is_empty() {
                let google_file_name = files[0]
                    .get("name")
                    .and_then(|id| id.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Failed to get file Name"))?;

                if google_file_name == file_name {
                    return Ok(());
                }
            }
        }

        let response = client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(access_token)
            .header("Content-Type", "multipart/related; boundary=foo_bar_baz")
            .body(format!(
                "--foo_bar_baz\r\n\
                 Content-Type: application/json; charset=UTF-8\r\n\r\n\
                 {{\"name\": \"{}\", \"parents\": [\"appDataFolder\"]}}\r\n\
                 --foo_bar_baz\r\n\
                 Content-Type: application/json\r\n\r\n\
                 {{\"is_primary\": true}}\r\n\
                 --foo_bar_baz--",
                file_name
            ))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to create secondary drive: {}",
                response.text().await?
            ))
        }
    }
}
