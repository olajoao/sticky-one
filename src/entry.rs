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

    pub fn parse(s: &str) -> Option<Self> {
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
                let collapsed: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_text_plain() {
        let e = Entry::new_text("hello world".into());
        assert_eq!(e.content_type, ContentType::Text);
        assert_eq!(e.content.as_deref(), Some("hello world"));
        assert!(e.image_data.is_none());
    }

    #[test]
    fn new_text_detects_link() {
        let e = Entry::new_text("https://example.com".into());
        assert_eq!(e.content_type, ContentType::Link);
    }

    #[test]
    fn new_text_not_link() {
        let e = Entry::new_text("just some text".into());
        assert_eq!(e.content_type, ContentType::Text);
    }

    #[test]
    fn new_image() {
        let data = vec![0x89, b'P', b'N', b'G', 1, 2, 3];
        let e = Entry::new_image(data.clone());
        assert_eq!(e.content_type, ContentType::Image);
        assert!(e.content.is_none());
        assert_eq!(e.image_data.unwrap(), data);
    }

    #[test]
    fn display_preview_text() {
        let e = Entry::new_text("hello world".into());
        assert_eq!(e.display_preview(80), "hello world");
    }

    #[test]
    fn display_preview_truncates() {
        let e = Entry::new_text("a".repeat(100));
        let preview = e.display_preview(10);
        assert!(preview.ends_with("..."));
        assert!(preview.len() <= 14); // 10 chars + "..."
    }

    #[test]
    fn display_preview_collapses_whitespace() {
        let e = Entry::new_text("hello\n  world\t\tfoo".into());
        assert_eq!(e.display_preview(80), "hello world foo");
    }

    #[test]
    fn display_preview_image() {
        let e = Entry::new_image(vec![0; 100]);
        assert_eq!(e.display_preview(80), "[Image: 100 bytes]");
    }

    #[test]
    fn hash_deterministic() {
        let h1 = hash_content(b"test data");
        let h2 = hash_content(b"test data");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_different_for_different_input() {
        let h1 = hash_content(b"aaa");
        let h2 = hash_content(b"bbb");
        assert_ne!(h1, h2);
    }

    #[test]
    fn content_type_roundtrip() {
        for ct in [ContentType::Text, ContentType::Link, ContentType::Image] {
            assert_eq!(ContentType::parse(ct.as_str()), Some(ct));
        }
    }
}
