use crate::config::MAX_IMAGE_SIZE_BYTES;
use crate::entry::Entry;
use crate::error::{Result, StickyError};
use std::process::Command;

pub enum ClipboardContent {
    Text(String),
    Image(Vec<u8>),
    Empty,
}

fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
}

pub fn read() -> Result<ClipboardContent> {
    if is_wayland() {
        read_wayland()
    } else {
        read_x11()
    }
}

fn read_wayland() -> Result<ClipboardContent> {
    // Try image first (before text, to avoid binary data as text)
    let output = Command::new("wl-paste")
        .args(["--no-newline", "--type", "image/png"])
        .output();

    if let Ok(out) = &output {
        if out.status.success() && !out.stdout.is_empty() {
            let size = out.stdout.len();
            if size > MAX_IMAGE_SIZE_BYTES {
                return Err(StickyError::ImageTooLarge {
                    size,
                    max: MAX_IMAGE_SIZE_BYTES,
                });
            }
            return Ok(ClipboardContent::Image(out.stdout.clone()));
        }
    }

    // Try text explicitly
    let output = Command::new("wl-paste")
        .args(["--no-newline", "--type", "text/plain"])
        .output();

    if let Ok(out) = output {
        if out.status.success() && !out.stdout.is_empty() {
            if let Ok(text) = String::from_utf8(out.stdout) {
                return Ok(ClipboardContent::Text(text));
            }
        }
    }

    Ok(ClipboardContent::Empty)
}

fn read_x11() -> Result<ClipboardContent> {
    // Try image first
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "image/png", "-o"])
        .output();

    if let Ok(out) = &output {
        if out.status.success() && !out.stdout.is_empty() {
            let size = out.stdout.len();
            if size > MAX_IMAGE_SIZE_BYTES {
                return Err(StickyError::ImageTooLarge {
                    size,
                    max: MAX_IMAGE_SIZE_BYTES,
                });
            }
            return Ok(ClipboardContent::Image(out.stdout.clone()));
        }
    }

    // Try text
    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-o"])
        .output();

    if let Ok(out) = output {
        if out.status.success() && !out.stdout.is_empty() {
            if let Ok(text) = String::from_utf8(out.stdout) {
                return Ok(ClipboardContent::Text(text));
            }
        }
    }

    Ok(ClipboardContent::Empty)
}

pub fn write_text(text: &str) -> Result<()> {
    if is_wayland() {
        write_text_wayland(text)
    } else {
        write_text_x11(text)
    }
}

fn write_text_wayland(text: &str) -> Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| StickyError::Clipboard(e.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| StickyError::Clipboard(e.to_string()))?;
    }

    child.wait().map_err(|e| StickyError::Clipboard(e.to_string()))?;
    Ok(())
}

fn write_text_x11(text: &str) -> Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| StickyError::Clipboard(e.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| StickyError::Clipboard(e.to_string()))?;
    }

    child.wait().map_err(|e| StickyError::Clipboard(e.to_string()))?;
    Ok(())
}

pub fn write_image(png_data: &[u8]) -> Result<()> {
    if is_wayland() {
        write_image_wayland(png_data)
    } else {
        write_image_x11(png_data)
    }
}

fn write_image_wayland(png_data: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("wl-copy")
        .args(["--type", "image/png"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| StickyError::Clipboard(e.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(png_data)
            .map_err(|e| StickyError::Clipboard(e.to_string()))?;
    }

    child.wait().map_err(|e| StickyError::Clipboard(e.to_string()))?;
    Ok(())
}

fn write_image_x11(png_data: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "image/png"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| StickyError::Clipboard(e.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(png_data)
            .map_err(|e| StickyError::Clipboard(e.to_string()))?;
    }

    child.wait().map_err(|e| StickyError::Clipboard(e.to_string()))?;
    Ok(())
}

pub fn write_entry(entry: &Entry) -> Result<()> {
    match entry.content_type {
        crate::entry::ContentType::Text | crate::entry::ContentType::Link => {
            if let Some(ref text) = entry.content {
                write_text(text)?;
            }
        }
        crate::entry::ContentType::Image => {
            if let Some(ref data) = entry.image_data {
                write_image(data)?;
            }
        }
    }
    Ok(())
}

pub fn read_as_entry() -> Result<Option<Entry>> {
    match read()? {
        ClipboardContent::Text(text) => Ok(Some(Entry::new_text(text))),
        ClipboardContent::Image(data) => Ok(Some(Entry::new_image(data))),
        ClipboardContent::Empty => Ok(None),
    }
}
