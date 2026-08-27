//! GitHub Contents API as durable JSON store (private repo).
//! Render holds zero data files — every row is a file under chess/ (GITHUB_DATA_ROOT).
//! Database ID: dstabase7837638362826373

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct GitHubStore {
    client: Client,
    owner: String,
    repo: String,
    token: String,
    branch: String,
    /// Path prefix in repo (default: "chess")
    root: String,
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("api: {0}")]
    Api(String),
    #[error("not found")]
    NotFound,
    #[error("conflict — retry")]
    Conflict,
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, serde::Deserialize)]
struct ContentsResponse {
    sha: String,
    content: Option<String>,
    encoding: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ContentsListItem {
    name: String,
    path: String,
    sha: String,
    #[serde(rename = "type")]
    kind: String,
}

impl GitHubStore {
    pub fn new(owner: String, repo: String, token: String, branch: String) -> Self {
        Self::with_root(owner, repo, token, branch, "chess".into())
    }

    pub fn with_root(owner: String, repo: String, token: String, branch: String, root: String) -> Self {
        Self {
            client: Client::new(),
            owner,
            repo,
            token,
            branch,
            root: root.trim_matches('/').to_string(),
        }
    }

    fn doc_path(&self, collection: &str, id: &str) -> String {
        format!("{}/{}/{}.json", self.root, collection, id)
    }

    fn collection_path(&self, collection: &str) -> String {
        format!("{}/{}", self.root, collection)
    }

    fn api_base(&self) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/contents",
            self.owner, self.repo
        )
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut h = reqwest::header::HeaderMap::new();
        h.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        h.insert(
            reqwest::header::USER_AGENT,
            "genius-clan-api".parse().unwrap(),
        );
        h.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github+json".parse().unwrap(),
        );
        h
    }

    /// Read JSON document at `{root}/{collection}/{id}.json`. Returns (value, sha).
    pub async fn get_json<T: DeserializeOwned>(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<(T, String), StoreError> {
        let path = self.doc_path(collection, id);
        let url = format!("{}/{}?ref={}", self.api_base(), path, self.branch);
        let res = self.client.get(&url).headers(self.headers()).send().await?;
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(StoreError::NotFound);
        }
        if !res.status().is_success() {
            return Err(StoreError::Api(res.text().await.unwrap_or_default()));
        }
        let body: ContentsResponse = res.json().await?;
        let raw = body.content.unwrap_or_default().replace('\n', "");
        let bytes = B64.decode(raw).map_err(|e| StoreError::Api(e.to_string()))?;
        let val: T = serde_json::from_slice(&bytes)?;
        Ok((val, body.sha))
    }

    /// Create or update JSON. Pass previous `sha` for update; `None` for create.
    pub async fn put_json<T: Serialize>(
        &self,
        collection: &str,
        id: &str,
        value: &T,
        sha: Option<&str>,
        message: &str,
    ) -> Result<String, StoreError> {
        let path = self.doc_path(collection, id);
        let url = format!("{}/{}", self.api_base(), path);
        let content = B64.encode(serde_json::to_vec_pretty(value)?);
        let mut payload = serde_json::json!({
            "message": message,
            "content": content,
            "branch": self.branch,
        });
        if let Some(s) = sha {
            payload["sha"] = Value::String(s.to_string());
        }
        let res = self
            .client
            .put(&url)
            .headers(self.headers())
            .json(&payload)
            .send()
            .await?;
        if res.status() == reqwest::StatusCode::CONFLICT {
            return Err(StoreError::Conflict);
        }
        if !res.status().is_success() {
            return Err(StoreError::Api(res.text().await.unwrap_or_default()));
        }
        let body: Value = res.json().await?;
        Ok(body["content"]["sha"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    pub async fn delete(
        &self,
        collection: &str,
        id: &str,
        sha: &str,
        message: &str,
    ) -> Result<(), StoreError> {
        let path = self.doc_path(collection, id);
        let url = format!("{}/{}", self.api_base(), path);
        let payload = serde_json::json!({
            "message": message,
            "sha": sha,
            "branch": self.branch,
        });
        let res = self
            .client
            .delete(&url)
            .headers(self.headers())
            .json(&payload)
            .send()
            .await?;
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }
        if !res.status().is_success() {
            return Err(StoreError::Api(res.text().await.unwrap_or_default()));
        }
        Ok(())
    }

    /// List file names (ids without .json) in a collection directory.
    pub async fn list_ids(&self, collection: &str) -> Result<Vec<String>, StoreError> {
        let path = self.collection_path(collection);
        let url = format!("{}/{}?ref={}", self.api_base(), path, self.branch);
        let res = self.client.get(&url).headers(self.headers()).send().await?;
        if res.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(vec![]);
        }
        if !res.status().is_success() {
            return Err(StoreError::Api(res.text().await.unwrap_or_default()));
        }
        let items: Vec<ContentsListItem> = res.json().await?;
        Ok(items
            .into_iter()
            .filter(|i| i.kind == "file" && i.name.ends_with(".json"))
            .map(|i| i.name.trim_end_matches(".json").to_string())
            .collect())
    }

    /// Index helpers: whole JSON map/list at chess/indexes/{name}.json
    pub async fn get_index<T: DeserializeOwned>(&self, name: &str) -> Result<(T, String), StoreError> {
        self.get_json("indexes", name).await
    }

    pub async fn put_index<T: Serialize>(
        &self,
        name: &str,
        value: &T,
        sha: Option<&str>,
        message: &str,
    ) -> Result<String, StoreError> {
        self.put_json("indexes", name, value, sha, message).await
    }
}

/// Shared handle placed on AppState when GitHub store is configured.
pub type SharedGitHubStore = Arc<GitHubStore>;
