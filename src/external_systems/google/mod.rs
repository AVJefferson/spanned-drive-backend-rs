pub mod config;

use reqwest::Response;

use self::config::GoogleConfig;

pub struct GoogleClient {
    pub config: GoogleConfig,
}

// ---------- Filename / query encoding helpers ----------
//
// These mirror the on-disk Google Drive appData naming conventions used by
// this service. They MUST stay byte-for-byte compatible with previously
// uploaded files, otherwise existing entries become unreadable.

fn encode_email_for_filename(email: &str) -> String {
    email.replace("@", "__at__").replace(".", "__dot__")
}

fn encode_drive_name(name: &str) -> String {
    name.replace("@", "__at__")
        .replace(".", "__dot__")
        .replace("/", "__slash__")
        .replace("\\", "__backslash__")
        .replace(" ", "__space__")
}

fn secondary_drive_filename(
    primary_email_encoded: &str,
    drive_provider: &str,
    secondary_email_encoded: &str,
) -> String {
    format!(
        "sdrive---secondary-drive---{}---{}---{}.json",
        primary_email_encoded, drive_provider, secondary_email_encoded
    )
}

fn logical_folder_filename(primary_email_encoded: &str, folder_name_encoded: &str) -> String {
    format!(
        "sdrive---logical-folder---{}---{}.json",
        primary_email_encoded, folder_name_encoded
    )
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

    /// Returns true iff an appData file with the given exact name already exists.
    async fn appdata_file_exists(
        &self,
        access_token: &str,
        file_name: &str,
    ) -> anyhow::Result<bool> {
        let existing = self
            .list_appdata_files(access_token.to_string(), format!("name = '{}'", file_name))
            .await?;

        if let Some(files) = existing.as_array() {
            if !files.is_empty() {
                let google_file_name = files[0]
                    .get("name")
                    .and_then(|n| n.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Failed to get file Name"))?;

                if google_file_name == file_name {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Uploads a new appData file with the given metadata name and JSON
    /// payload via the multipart upload endpoint. The caller is responsible
    /// for inspecting the returned `Response` status.
    async fn upload_appdata_file(
        &self,
        access_token: &str,
        file_name: &str,
        content_json: &str,
    ) -> anyhow::Result<Response> {
        let client = reqwest::Client::new();

        let body = format!(
            "--foo_bar_baz\r\n\
             Content-Type: application/json; charset=UTF-8\r\n\r\n\
             {{\"name\": \"{}\", \"parents\": [\"appDataFolder\"]}}\r\n\
             --foo_bar_baz\r\n\
             Content-Type: application/json\r\n\r\n\
             {}\r\n\
             --foo_bar_baz--",
            file_name, content_json
        );

        let response = client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(access_token)
            .header("Content-Type", "multipart/related; boundary=foo_bar_baz")
            .body(body)
            .send()
            .await?;

        Ok(response)
    }

    pub async fn set_is_primary(&self, access_token: String) -> anyhow::Result<()> {
        // check first if the file already exists to avoid unnecessary uploads
        if self
            .appdata_file_exists(&access_token, "is_primary.json")
            .await?
        {
            return Ok(());
        }

        let response = self
            .upload_appdata_file(&access_token, "is_primary.json", "{\"is_primary\": true}")
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
        let primary_email = encode_email_for_filename(primary_email);

        let response: reqwest::Response = client
            .get(format!("https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name contains 'sdrive---secondary-drive---{}---'", primary_email.clone()))
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
        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email == "" {
            return Err(anyhow::anyhow!("Failed to fetch user info"));
        }
        let primary_email = encode_email_for_filename(primary_email);

        let file_name = secondary_drive_filename(
            &primary_email,
            &drive_provider,
            &encode_email_for_filename(&new_secondary_drive_email),
        );

        if self.appdata_file_exists(&access_token, &file_name).await? {
            return Ok(());
        }

        let response = self
            .upload_appdata_file(&access_token, &file_name, "{\"new_file\": true}")
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

    pub async fn get_logical_folders(&self, access_token: String) -> anyhow::Result<Vec<String>> {
        let client = reqwest::Client::new();

        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email == "" {
            return Ok(vec![]);
        }
        let primary_email = encode_email_for_filename(primary_email);

        let response: reqwest::Response = client
            .get(format!("https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name contains 'sdrive---logical-folder---{}---'", primary_email.clone()))
            .bearer_auth(access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        let mut logical_folders = Vec::new();

        if let Some(files) = json.get("files").and_then(|f| f.as_array()) {
            for file in files {
                if let Some(id) = file.get("id").and_then(|id| id.as_str()) {
                    if let Some(name) = file.get("name").and_then(|n| n.as_str()) {
                        logical_folders.push(format!(
                            "{}{}",
                            id,
                            name.to_string()
                                .replace("sdrive---logical-folder---", "")
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

        Ok(logical_folders)
    }

    pub async fn set_logical_folder(
        &self,
        access_token: String,
        drive_name: String,
        drives: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email == "" {
            return Err(anyhow::anyhow!("Failed to fetch user info"));
        }
        let primary_email = encode_email_for_filename(primary_email);

        // Watch out for special characters in drive_name that might cause
        // issues with file naming in Google Drive.
        let logical_folder_name = encode_drive_name(&drive_name);

        let file_name = logical_folder_filename(&primary_email, &logical_folder_name);

        if self.appdata_file_exists(&access_token, &file_name).await? {
            return Ok(());
        }

        let content = serde_json::json!({
            "new_file": true,
            "drives": drives,
        });

        let response = self
            .upload_appdata_file(&access_token, &file_name, &content.to_string())
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to create logical folder: {}",
                response.text().await?
            ))
        }
    }
}
