//! Load bearer tokens from `allowed_clients/<name>.key` JSON files.

use dashmap::DashMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct ClientKeyFile {
    pub token: String,
    pub permissions: Vec<String>,
}

struct LoadedEntry {
    path: PathBuf,
    key_name: String,
    token: String,
    permissions: Vec<String>,
}

fn token_display(token: &str) -> String {
    if token.len() > 10 {
        format!(
            "{}...{}",
            &token[..5],
            &token[token.len() - 5..]
        )
    } else {
        token.to_string()
    }
}

#[cfg(unix)]
fn is_secure_key_mode(mode: u32) -> bool {
    let masked = mode & 0o777;
    masked == 0o600 || masked == 0o400
}

pub async fn load_into_map(auth_tokens: &DashMap<String, Vec<String>>) -> std::io::Result<()> {
    #[cfg(not(unix))]
    {
        static SKIP_PERM_WARN: std::sync::Once = std::sync::Once::new();
        SKIP_PERM_WARN.call_once(|| {
            log::warn!("allowed_clients: Unix permission checks are skipped on this platform");
        });
    }

    let mut read_dir = match tokio::fs::read_dir("allowed_clients").await {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            log::warn!("'allowed_clients' directory not found. No client keys will be loaded.");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let mut loaded: Vec<LoadedEntry> = Vec::new();

    while let Some(entry) = read_dir.next_entry().await? {
        let path: PathBuf = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension() else {
            continue;
        };

        if extension == "key" {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let meta = tokio::fs::metadata(&path).await?;
                let mode = meta.permissions().mode();
                if !is_secure_key_mode(mode) {
                    log::error!(
                        "allowed_clients: {:?} has insecure mode {:o}; require 600 or 400 — skipping",
                        path.file_name().unwrap().to_string_lossy(),
                        mode & 0o7777
                    );
                    continue;
                }
            }
            let key_name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();

            let content = match tokio::fs::read_to_string(&path).await {
                Ok(c) => c,
                Err(e) => {
                    log::error!("allowed_clients: failed to read {:?}: {}", path, e);
                    continue;
                }
            };

            let parsed: ClientKeyFile = match serde_json::from_str(&content) {
                Ok(p) => p,
                Err(e) => {
                    log::error!(
                        "allowed_clients: invalid JSON in {:?} for key name {:?}: {}",
                        path,
                        key_name,
                        e
                    );
                    continue;
                }
            };

            if parsed.token.is_empty() {
                log::error!(
                    "allowed_clients: empty token in {:?} (key name {:?}) — skipping",
                    path,
                    key_name
                );
                continue;
            }

            loaded.push(LoadedEntry {
                path,
                key_name,
                token: parsed.token,
                permissions: parsed.permissions,
            });
        } else if extension == "blocked" || extension == "deleted" || extension == "disabled" {
            let key = path.file_name().unwrap().to_string_lossy().into_owned();
            let display_key = if key.len() > 10 {
                format!("{}...{}", &key[..5], &key[key.len() - 5..])
            } else {
                key
            };
            log::warn!("key: {} is {}", display_key, extension.to_string_lossy());
        }
    }

    let mut by_token: HashMap<String, Vec<LoadedEntry>> = HashMap::new();
    for entry in loaded {
        by_token.entry(entry.token.clone()).or_default().push(entry);
    }

    for (token, mut entries) in by_token {
        if entries.len() > 1 {
            entries.sort_by(|a, b| a.path.cmp(&b.path));
            let paths: Vec<String> = entries
                .iter()
                .map(|e| e.path.display().to_string())
                .collect();
            log::error!(
                "allowed_clients: duplicate token {} in {} key files — not loading any of them: {}",
                token_display(&token),
                entries.len(),
                paths.join(", ")
            );
            continue;
        }

        let entry = entries.pop().expect("len == 1");
        auth_tokens.insert(entry.token.clone(), entry.permissions.clone());

        log::info!(
            "allowed_clients: loaded key name {:?} → token {} with permissions {:?}",
            entry.key_name,
            token_display(&entry.token),
            entry.permissions
        );
    }

    Ok(())
}
