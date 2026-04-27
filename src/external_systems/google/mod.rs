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

    pub async fn get_appdata_file(
        &self,
        access_token: String,
        file_id: String,
    ) -> anyhow::Result<String> {
        let client = reqwest::Client::new();
        let response: reqwest::Response = client
            .get(format!(
                "https://www.googleapis.com/drive/v3/files/{}?alt=media",
                file_id
            ))
            .bearer_auth(access_token)
            .send()
            .await?;
        let json = response.json::<serde_json::Value>().await?;

        Ok(json.to_string())
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

    pub async fn get_secondary_drives(
        &self,
        access_token: String,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let client = reqwest::Client::new();

        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email.is_empty() {
            return Ok(vec![]);
        }
        let primary_email_enc = encode_email_for_filename(primary_email);

        let response: reqwest::Response = client
            .get(format!(
                "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name%20contains%20'sdrive---secondary-drive---{}---'",
                primary_email_enc
            ))
            .bearer_auth(access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        let mut secondary_drives = Vec::new();

        if let Some(files) = json.get("files").and_then(|f| f.as_array()) {
            for file in files {
                let file_id = match file.get("id").and_then(|v| v.as_str()) {
                    Some(id) => id,
                    None => continue,
                };
                let raw_name = match file.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => continue,
                };

                // filename: sdrive---secondary-drive---{primary_enc}---{provider}---{email_enc}.json
                let stripped = raw_name
                    .replace("sdrive---secondary-drive---", "")
                    .replace(&format!("{}---", primary_email_enc), "")
                    .replace(".json", "");

                // stripped is now: {provider}---{email_enc}
                let parts: Vec<&str> = stripped.splitn(2, "---").collect();
                if parts.len() != 2 {
                    continue;
                }
                let provider = parts[0].to_string();
                let email = parts[1].replace("__at__", "@").replace("__dot__", ".");

                secondary_drives.push(serde_json::json!({
                    "file_id": file_id,
                    "provider": provider,
                    "email": email,
                }));
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

    /// Returns structured logical-folder records. Each record contains the
    /// appData file id, the human-readable folder name, and the drives list
    /// (tuples of arbitrary length, forwarded verbatim from the stored JSON).
    pub async fn get_logical_folders(
        &self,
        access_token: String,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let client = reqwest::Client::new();

        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email.is_empty() {
            return Ok(vec![]);
        }
        let primary_email_enc = encode_email_for_filename(primary_email);

        let response: reqwest::Response = client
            .get(format!(
                "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name%20contains%20'sdrive---logical-folder---{}---'",
                primary_email_enc
            ))
            .bearer_auth(access_token.clone())
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        let mut results = Vec::new();

        if let Some(files) = json.get("files").and_then(|f| f.as_array()) {
            for file in files {
                let file_id = match file.get("id").and_then(|v| v.as_str()) {
                    Some(id) => id,
                    None => continue,
                };
                let raw_name = match file.get("name").and_then(|v| v.as_str()) {
                    Some(n) => n,
                    None => continue,
                };

                // Decode folder name: strip prefix + primary email encoding then
                // reverse the individual character substitutions.
                let folder_name = raw_name
                    .replace("sdrive---logical-folder---", "")
                    .replace(&format!("{}---", primary_email_enc), "")
                    .replace(".json", "")
                    .replace("__at__", "@")
                    .replace("__dot__", ".")
                    .replace("__slash__", "/")
                    .replace("__backslash__", "\\")
                    .replace("__space__", " ");

                // Read the file content to get the drives list.
                let content_result = self
                    .get_appdata_file(access_token.clone(), file_id.to_string())
                    .await;

                let drives = match content_result {
                    Ok(content_str) => serde_json::from_str::<serde_json::Value>(&content_str)
                        .ok()
                        .and_then(|v| v.get("drives").cloned())
                        .unwrap_or(serde_json::Value::Array(vec![])),
                    Err(_) => serde_json::Value::Array(vec![]),
                };

                results.push(serde_json::json!({
                    "file_id": file_id,
                    "name": folder_name,
                    "drives": drives,
                }));
            }
        }

        Ok(results)
    }

    pub async fn set_logical_folder(
        &self,
        access_token: String,
        drive_name: String,
        drives: Vec<Vec<String>>,
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

    // ---- Drive operation proxies ----

    const DRIVE_FIELDS: &'static str =
        "id,name,mimeType,size,parents,modifiedTime,webViewLink,webContentLink,thumbnailLink";
    const FOLDER_MIME: &'static str = "application/vnd.google-apps.folder";

    pub async fn drive_about(&self, access_token: String) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/drive/v3/about?fields=storageQuota")
            .bearer_auth(&access_token)
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        let quota = json
            .get("storageQuota")
            .cloned()
            .unwrap_or(serde_json::Value::Object(Default::default()));

        let total = quota
            .get("limit")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let used = quota
            .get("usageInDrive")
            .or_else(|| quota.get("usage"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        let app_data_usage = quota
            .get("usageInDriveTrash")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(serde_json::json!({
            "totalSpace": total,
            "usedSpace": used,
            "freeSpace": total.saturating_sub(used),
            "appDataUsage": app_data_usage,
        }))
    }

    pub async fn list_drive_children(
        &self,
        access_token: String,
        parent_id: String,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let query = format!("'{}' in parents and trashed = false", parent_id);
        let fields = format!("files({})", Self::DRIVE_FIELDS);
        let url = format!(
            "https://www.googleapis.com/drive/v3/files?q={}&fields={}&orderBy=folder,name_natural",
            urlencoding::encode(&query),
            urlencoding::encode(&fields),
        );

        let response = client.get(&url).bearer_auth(&access_token).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Drive list_children {}: {}", status, text));
        }

        let json = response.json::<serde_json::Value>().await?;
        let files = json
            .get("files")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));

        Ok(Self::map_drive_items(files))
    }

    pub async fn get_file_metadata_rich(
        &self,
        access_token: String,
        file_id: String,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}?fields={}",
            urlencoding::encode(&file_id),
            urlencoding::encode(Self::DRIVE_FIELDS),
        );

        let response = client.get(&url).bearer_auth(&access_token).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Drive file_metadata {}: {}", status, text));
        }

        let item = response.json::<serde_json::Value>().await?;
        Ok(Self::map_drive_item(&item))
    }

    pub async fn create_drive_folder(
        &self,
        access_token: String,
        name: String,
        parent_id: String,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "name": name,
            "mimeType": Self::FOLDER_MIME,
            "parents": [parent_id],
        });

        let response = client
            .post("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(&access_token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Drive create_folder {}: {}", status, text));
        }

        let item = response.json::<serde_json::Value>().await?;
        Ok(Self::map_drive_item(&item))
    }

    pub async fn upload_drive_file(
        &self,
        access_token: String,
        file_name: String,
        mime_type: String,
        data: Vec<u8>,
        parent_id: String,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let boundary = format!("sdrive-{}", uuid_v4());

        let metadata = serde_json::json!({
            "name": file_name,
            "parents": [parent_id],
        })
        .to_string();

        let body = format!(
            "--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n\
             --{boundary}\r\nContent-Type: {mime_type}\r\n\r\n"
        );

        let mut body_bytes = body.into_bytes();
        body_bytes.extend_from_slice(&data);
        body_bytes.extend_from_slice(format!("\r\n--{boundary}--").as_bytes());

        let response = client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(&access_token)
            .header(
                "Content-Type",
                format!("multipart/related; boundary={boundary}"),
            )
            .body(body_bytes)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Drive upload_file {}: {}", status, text));
        }

        let item = response.json::<serde_json::Value>().await?;
        Ok(Self::map_drive_item(&item))
    }

    pub async fn delete_drive_item(
        &self,
        access_token: String,
        file_id: String,
    ) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}",
            urlencoding::encode(&file_id),
        );

        let response = client
            .delete(&url)
            .bearer_auth(&access_token)
            .send()
            .await?;

        if !response.status().is_success() && response.status().as_u16() != 204 {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Drive delete_item {}: {}", status, text));
        }

        Ok(())
    }

    pub async fn copy_drive_item(
        &self,
        access_token: String,
        file_id: String,
        parent_id: String,
        name: Option<String>,
    ) -> anyhow::Result<serde_json::Value> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}/copy",
            urlencoding::encode(&file_id),
        );

        let mut body = serde_json::json!({ "parents": [parent_id] });
        if let Some(n) = name {
            body["name"] = serde_json::Value::String(n);
        }

        let response = client
            .post(&url)
            .bearer_auth(&access_token)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Drive copy_item {}: {}", status, text));
        }

        let item = response.json::<serde_json::Value>().await?;
        Ok(Self::map_drive_item(&item))
    }

    // ---- private helpers ----

    fn map_drive_item(item: &serde_json::Value) -> serde_json::Value {
        let mime = item
            .get("mimeType")
            .and_then(|v| v.as_str())
            .unwrap_or("application/octet-stream");

        serde_json::json!({
            "id": item.get("id").and_then(|v| v.as_str()).unwrap_or(""),
            "name": item.get("name").and_then(|v| v.as_str()).unwrap_or(""),
            "mimeType": mime,
            "size": item.get("size").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()),
            "parents": item.get("parents").cloned().unwrap_or(serde_json::Value::Array(vec![])),
            "modifiedTime": item.get("modifiedTime").and_then(|v| v.as_str()),
            "webViewLink": item.get("webViewLink").and_then(|v| v.as_str()),
            "webContentLink": item.get("webContentLink").and_then(|v| v.as_str()),
            "thumbnailLink": item.get("thumbnailLink").and_then(|v| v.as_str()),
            "isFolder": mime == Self::FOLDER_MIME,
        })
    }

    fn map_drive_items(files: serde_json::Value) -> serde_json::Value {
        match files {
            serde_json::Value::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(Self::map_drive_item).collect())
            }
            _ => serde_json::Value::Array(vec![]),
        }
    }

    // ---- appData by-name helpers (for browser-routed JSON state files) ----

    pub async fn get_appdata_file_by_name(
        &self,
        access_token: String,
        file_name: String,
    ) -> anyhow::Result<Option<String>> {
        let files = self
            .list_appdata_files(
                access_token.clone(),
                format!("name = '{}'", file_name.replace('\'', "\\'")),
            )
            .await?;

        let file_id = match files
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|f| f.get("id"))
            .and_then(|id| id.as_str())
        {
            Some(id) => id.to_string(),
            None => return Ok(None),
        };

        let content = self.get_appdata_file(access_token, file_id).await?;
        Ok(Some(content))
    }

    async fn update_appdata_file(
        &self,
        access_token: &str,
        file_id: &str,
        content_json: &str,
    ) -> anyhow::Result<()> {
        let client = reqwest::Client::new();
        let response = client
            .patch(format!(
                "https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=media",
                urlencoding::encode(file_id)
            ))
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .body(content_json.to_string())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Failed to update appdata file: {}",
                response.text().await?
            ))
        }
    }

    pub async fn set_appdata_file_by_name(
        &self,
        access_token: String,
        file_name: String,
        content_json: String,
    ) -> anyhow::Result<()> {
        let files = self
            .list_appdata_files(
                access_token.clone(),
                format!("name = '{}'", file_name.replace('\'', "\\'")),
            )
            .await?;

        if let Some(existing_id) = files
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|f| f.get("id"))
            .and_then(|id| id.as_str())
        {
            self.update_appdata_file(&access_token, existing_id, &content_json)
                .await?;
        } else {
            let response = self
                .upload_appdata_file(&access_token, &file_name, &content_json)
                .await?;
            if !response.status().is_success() {
                return Err(anyhow::anyhow!(
                    "Failed to create appdata file: {}",
                    response.text().await?
                ));
            }
        }

        Ok(())
    }

    // ---- File download proxy ----

    pub async fn download_drive_file_bytes(
        &self,
        access_token: String,
        file_id: String,
    ) -> anyhow::Result<Vec<u8>> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "https://www.googleapis.com/drive/v3/files/{}?alt=media",
                urlencoding::encode(&file_id)
            ))
            .bearer_auth(&access_token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Drive download_file {}: {}", status, text));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", t)
}
