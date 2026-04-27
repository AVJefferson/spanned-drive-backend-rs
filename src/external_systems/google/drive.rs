use super::{GoogleClient, helpers::timestamp_hex};

impl GoogleClient {
    const DRIVE_FIELDS: &'static str =
        "id,name,mimeType,size,parents,modifiedTime,webViewLink,webContentLink,thumbnailLink";
    const FOLDER_MIME: &'static str = "application/vnd.google-apps.folder";

    pub async fn drive_about(&self, access_token: String) -> anyhow::Result<serde_json::Value> {
        let response = self
            .http_client
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
        let query = format!("'{}' in parents and trashed = false", parent_id);
        let fields = format!("files({})", Self::DRIVE_FIELDS);
        let url = format!(
            "https://www.googleapis.com/drive/v3/files?q={}&fields={}&orderBy=folder,name_natural",
            urlencoding::encode(&query),
            urlencoding::encode(&fields),
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await?;

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
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}?fields={}",
            urlencoding::encode(&file_id),
            urlencoding::encode(Self::DRIVE_FIELDS),
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&access_token)
            .send()
            .await?;

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
        let body = serde_json::json!({
            "name": name,
            "mimeType": Self::FOLDER_MIME,
            "parents": [parent_id],
        });

        let response = self
            .http_client
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
        let boundary = format!("sdrive-{}", timestamp_hex());

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

        let response = self
            .http_client
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
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}",
            urlencoding::encode(&file_id),
        );

        let response = self
            .http_client
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
        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}/copy",
            urlencoding::encode(&file_id),
        );

        let mut body = serde_json::json!({ "parents": [parent_id] });
        if let Some(n) = name {
            body["name"] = serde_json::Value::String(n);
        }

        let response = self
            .http_client
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

    pub async fn download_drive_file_bytes(
        &self,
        access_token: String,
        file_id: String,
    ) -> anyhow::Result<Vec<u8>> {
        let response = self
            .http_client
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
}
