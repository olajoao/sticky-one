use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Link,
    Image,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Link => "link",
            Self::Image => "image",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(Self::Text),
            "link" => Some(Self::Link),
            "image" => Some(Self::Image),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: i64,
    pub content_type: ContentType,
    pub content: Option<String>,
    pub image_data: Option<Vec<u8>>,
    pub hash: String,
    pub created_at: i64,
}

impl Entry {
    pub fn new_text(text: String) -> Self {
        let content_type = if is_url(&text) {
            ContentType::Link
        } else {
            ContentType::Text
        };
        let hash = hash_content(text.as_bytes());

        Self {
            id: 0,
            content_type,
            content: Some(text),
            image_data: None,
            hash,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn new_image(data: Vec<u8>) -> Self {
        let hash = hash_content(&data);

        Self {
            id: 0,
            content_type: ContentType::Image,
            content: None,
            image_data: Some(data),
            hash,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn display_preview(&self, max_len: usize) -> String {
        match self.content_type {
            ContentType::Text | ContentType::Link => {
                let text = self.content.as_deref().unwrap_or("");
                // Collapse whitespace/newlines to single space
                let collapsed: String = text
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                // Safe truncation at char boundary
                if collapsed.len() > max_len {
                    let truncated: String = collapsed.chars().take(max_len).collect();
                    format!("{}...", truncated)
                } else {
                    collapsed
                }
            }
            ContentType::Image => {
                let size = self.image_data.as_ref().map(|d| d.len()).unwrap_or(0);
                format!("[Image: {} bytes]", size)
            }
        }
    }
}

fn is_url(text: &str) -> bool {
    Url::parse(text.trim()).is_ok()
}

fn hash_content(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}
