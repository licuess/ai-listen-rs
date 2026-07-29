use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SessionStore {
    root: PathBuf,
}

impl SessionStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn create_session(&self, title: &str) -> Result<PathBuf, String> {
        let session = self.session_path(title);
        fs::create_dir_all(&session).map_err(|error| error.to_string())?;

        let note = session.join("notes.md");
        if !note.exists() {
            fs::write(&note, format!("# {title}\n\n")).map_err(|error| error.to_string())?;
        }

        Ok(session)
    }

    pub fn ensure_session(&self, title: &str) -> Result<PathBuf, String> {
        let session = self.session_path(title);
        if session.exists() {
            Ok(session)
        } else {
            self.create_session(title)
        }
    }

    pub fn append_note(&self, title: &str, content: &str) -> Result<PathBuf, String> {
        let session = self.ensure_session(title)?;
        let note = session.join("notes.md");
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&note)
            .map_err(|error| error.to_string())?;

        writeln!(file, "- {}", content.trim()).map_err(|error| error.to_string())?;
        Ok(note)
    }

    pub fn list_sessions(&self) -> Result<Vec<PathBuf>, String> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut sessions = fs::read_dir(&self.root)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();

        sessions.sort();
        Ok(sessions)
    }

    fn session_path(&self, title: &str) -> PathBuf {
        self.root.join(slugify(title))
    }
}

pub fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for ch in title.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            slug.push(ch);
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "untitled".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn slugifies_titles() {
        assert_eq!(slugify("Product Sync 2026"), "product-sync-2026");
        assert_eq!(slugify("  AI 听记  "), "ai-听记");
        assert_eq!(slugify("会议"), "会议");
    }
}
