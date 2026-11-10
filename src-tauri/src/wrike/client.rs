use serde::Deserialize;

/// Wrike API v4 client. Host is per-account (returned by OAuth token
/// exchange); for a permanent token we default to `www.wrike.com`, which is
/// Wrike's documented default entry point. If a given account actually lives
/// on a different host, the first request below will fail clearly and the
/// host can be corrected.
pub struct WrikeClient {
    http: reqwest::Client,
    host: String,
    token: String,
}

#[derive(Debug)]
pub enum WrikeError {
    InvalidToken,
    RateLimited { retry_after_secs: Option<u64> },
    Network(String),
    Api { status: u16, body: String },
}

impl std::fmt::Display for WrikeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WrikeError::InvalidToken => {
                write!(f, "Your Wrike token looks invalid or expired. Please reconnect.")
            }
            WrikeError::RateLimited { retry_after_secs } => match retry_after_secs {
                Some(s) => write!(f, "Wrike rate limit reached. Try again in {s}s."),
                None => write!(f, "Wrike rate limit reached. Try again shortly."),
            },
            WrikeError::Network(e) => write!(f, "Could not reach Wrike: {e}"),
            WrikeError::Api { status, body } => write!(f, "Wrike API error ({status}): {body}"),
        }
    }
}

#[derive(Deserialize)]
struct Envelope<T> {
    data: Vec<T>,
}

#[derive(Deserialize)]
pub struct Contact {
    pub id: String,
    #[serde(default, rename = "firstName")]
    pub first_name: String,
    #[serde(default, rename = "lastName")]
    pub last_name: String,
}

#[derive(Deserialize)]
pub struct Space {
    pub id: String,
    pub title: String,
}

#[derive(Deserialize)]
pub struct Folder {
    pub id: String,
    pub title: String,
    #[serde(default, rename = "parentIds")]
    pub parent_ids: Vec<String>,
}

#[derive(Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(default, rename = "responsibleIds")]
    pub responsible_ids: Vec<String>,
    #[serde(default, rename = "parentIds")]
    pub parent_ids: Vec<String>,
}

#[derive(Deserialize)]
struct TasksPage {
    data: Vec<Task>,
    #[serde(default, rename = "nextPageToken")]
    next_page_token: Option<String>,
}

impl WrikeClient {
    pub fn new(host: &str, token: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            host: host.to_string(),
            token: token.to_string(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("https://{}/api/v4{}", self.host, path)
    }

    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T, WrikeError> {
        let resp = self
            .http
            .get(self.url(path))
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| WrikeError::Network(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(WrikeError::InvalidToken);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after_secs = resp
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());
            return Err(WrikeError::RateLimited { retry_after_secs });
        }
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(WrikeError::Api {
                status: status.as_u16(),
                body,
            });
        }
        resp.json::<T>()
            .await
            .map_err(|e| WrikeError::Network(e.to_string()))
    }

    pub async fn whoami(&self) -> Result<Contact, WrikeError> {
        let envelope: Envelope<Contact> = self.get("/contacts?me=true").await?;
        envelope
            .data
            .into_iter()
            .next()
            .ok_or(WrikeError::Api {
                status: 200,
                body: "no contact returned for me=true".to_string(),
            })
    }

    pub async fn list_spaces(&self) -> Result<Vec<Space>, WrikeError> {
        let envelope: Envelope<Space> = self.get("/spaces").await?;
        Ok(envelope.data)
    }

    pub async fn list_space_folders(&self, space_id: &str) -> Result<Vec<Folder>, WrikeError> {
        let envelope: Envelope<Folder> =
            self.get(&format!("/spaces/{space_id}/folders")).await?;
        Ok(envelope.data)
    }

    /// Direct subfolders of `folder_id` — used to resolve one level of
    /// "project" breadcrumb underneath a client folder.
    pub async fn list_folder_subfolders(&self, folder_id: &str) -> Result<Vec<Folder>, WrikeError> {
        let envelope: Envelope<Folder> = self.get(&format!("/folders/{folder_id}/folders")).await?;
        Ok(envelope.data)
    }

    /// Tasks in `folder_id` and all of its descendant folders/projects.
    pub async fn list_folder_tasks(&self, folder_id: &str) -> Result<Vec<Task>, WrikeError> {
        let mut all = Vec::new();
        let mut path = format!("/folders/{folder_id}/tasks?descendants=true&pageSize=1000");
        loop {
            let page: TasksPage = self.get(&path).await?;
            all.extend(page.data);
            match page.next_page_token {
                Some(token) if !token.is_empty() => {
                    path = format!(
                        "/folders/{folder_id}/tasks?descendants=true&pageSize=1000&nextPageToken={token}"
                    );
                }
                _ => break,
            }
        }
        Ok(all)
    }
}
