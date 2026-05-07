use super::{
    GoogleClient,
    helpers::{
        encode_drive_name, encode_email_for_filename, logical_folder_filename,
        secondary_drive_filename,
    },
};

use reqwest::Response;

impl GoogleClient {
    pub async fn list_appdata_files(
        &self,
        access_token: String,
        query: String,
    ) -> anyhow::Result<serde_json::Value> {
        let response = self
            .http_client
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
        let response = self
            .http_client
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
        let response = self
            .http_client
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

    pub async fn set_is_primary(&self, access_token: String) -> anyhow::Result<()> {
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
        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email.is_empty() {
            return Ok(vec![]);
        }
        let primary_email_enc = encode_email_for_filename(primary_email);

        let response = self
            .http_client
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
        if primary_email.is_empty() {
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

    pub async fn get_logical_folders(
        &self,
        access_token: String,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let user = self.fetch_user_info(access_token.clone()).await?;
        let primary_email = user.get("email").and_then(|e| e.as_str()).unwrap_or("");
        if primary_email.is_empty() {
            return Ok(vec![]);
        }
        let primary_email_enc = encode_email_for_filename(primary_email);

        let response = self
            .http_client
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

                let folder_name = raw_name
                    .replace("sdrive---logical-folder---", "")
                    .replace(&format!("{}---", primary_email_enc), "")
                    .replace(".json", "")
                    .replace("__at__", "@")
                    .replace("__dot__", ".")
                    .replace("__slash__", "/")
                    .replace("__backslash__", "\\")
                    .replace("__space__", " ");

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
        if primary_email.is_empty() {
            return Err(anyhow::anyhow!("Failed to fetch user info"));
        }
        let primary_email = encode_email_for_filename(primary_email);

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

    // ---- private helpers ----

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

    async fn upload_appdata_file(
        &self,
        access_token: &str,
        file_name: &str,
        content_json: &str,
    ) -> anyhow::Result<Response> {
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

        let response = self
            .http_client
            .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(access_token)
            .header("Content-Type", "multipart/related; boundary=foo_bar_baz")
            .body(body)
            .send()
            .await?;

        Ok(response)
    }

    async fn update_appdata_file(
        &self,
        access_token: &str,
        file_id: &str,
        content_json: &str,
    ) -> anyhow::Result<()> {
        let response = self
            .http_client
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
}
