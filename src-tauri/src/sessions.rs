use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SessionStore {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionDetails {
    pub title: String,
    pub slug: String,
    pub path: String,
    pub notes: String,
    pub screenshots: Vec<String>,
    pub recordings: Vec<String>,
    pub audio: Vec<String>,
    pub transcripts: Vec<String>,
    pub is_recording: bool,
    pub is_audio_recording: bool,
}

impl SessionStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn create_session(&self, title: &str) -> Result<SessionDetails, String> {
        let slug = slugify(title);
        let session = self.root.join(&slug);
        fs::create_dir_all(&session).map_err(|error| error.to_string())?;

        let note = session.join("notes.md");
        if !note.exists() {
            fs::write(&note, format!("# {title}\n\n")).map_err(|error| error.to_string())?;
        }

        self.read_session(&slug)
    }

    pub fn read_session(&self, slug: &str) -> Result<SessionDetails, String> {
        let session = self.session_dir_by_slug(slug)?;
        let note_path = session.join("notes.md");
        let notes = fs::read_to_string(&note_path).unwrap_or_default();
        let title = notes
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .unwrap_or(slug)
            .to_string();

        let mut screenshots = Vec::new();
        let mut recordings = Vec::new();
        let mut audio = Vec::new();
        let mut transcripts = Vec::new();

        for entry in fs::read_dir(&session).map_err(|error| error.to_string())? {
            let path = entry.map_err(|error| error.to_string())?.path();
            let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            if name.ends_with(".png") {
                screenshots.push(path.to_string_lossy().to_string());
            } else if name.ends_with(".mp4") || name.ends_with(".mov") || name.ends_with(".mkv") {
                recordings.push(path.to_string_lossy().to_string());
            } else if name.ends_with(".wav") || name.ends_with(".mp3") || name.ends_with(".m4a") {
                audio.push(path.to_string_lossy().to_string());
            } else if name.starts_with("transcript-") && name.ends_with(".md") {
                transcripts.push(path.to_string_lossy().to_string());
            }
        }

        screenshots.sort();
        recordings.sort();
        audio.sort();
        transcripts.sort();

        Ok(SessionDetails {
            title,
            slug: slug.to_string(),
            path: session.to_string_lossy().to_string(),
            notes,
            screenshots,
            recordings,
            audio,
            transcripts,
            is_recording: session.join(".recording.pid").exists(),
            is_audio_recording: session.join(".audio.pid").exists(),
        })
    }

    pub fn save_note(&self, slug: &str, content: &str) -> Result<SessionDetails, String> {
        let session = self.session_dir_by_slug(slug)?;
        fs::write(session.join("notes.md"), content).map_err(|error| error.to_string())?;
        self.read_session(slug)
    }

    pub fn append_note(&self, slug: &str, content: &str) -> Result<SessionDetails, String> {
        let session = self.session_dir_by_slug(slug)?;
        let note_path = session.join("notes.md");
        let mut notes = fs::read_to_string(&note_path).unwrap_or_default();
        notes.push_str(content);
        fs::write(&note_path, &notes).map_err(|error| error.to_string())?;
        self.read_session(slug)
    }

    pub fn append_transcript(
        &self,
        slug: &str,
        transcript: &str,
    ) -> Result<SessionDetails, String> {
        let session = self.session_dir_by_slug(slug)?;
        let file = session.join(format!("transcript-{}.md", timestamp()));
        fs::write(&file, transcript).map_err(|error| error.to_string())?;

        let note_path = session.join("notes.md");
        let mut notes = fs::read_to_string(&note_path).unwrap_or_default();
        if !notes.ends_with('\n') {
            notes.push('\n');
        }
        notes.push_str("\n## 转写\n\n");
        notes.push_str(transcript.trim());
        notes.push('\n');
        fs::write(note_path, notes).map_err(|error| error.to_string())?;

        self.read_session(slug)
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionDetails>, String> {
        if !self.root.exists() {
            return Ok(Vec::new());
        }

        let mut slugs = fs::read_dir(&self.root)
            .map_err(|error| error.to_string())?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .filter_map(|path| path.file_name()?.to_str().map(str::to_string))
            .collect::<Vec<_>>();

        slugs.sort();
        slugs
            .into_iter()
            .map(|slug| self.read_session(&slug))
            .collect()
    }

    pub fn session_dir_by_slug(&self, slug: &str) -> Result<PathBuf, String> {
        if slug.contains("..") || slug.contains('\\') || slug.contains('/') {
            return Err("invalid session slug".to_string());
        }

        let session = self.root.join(slug);
        if session.exists() {
            Ok(session)
        } else {
            Err(format!("session not found: {slug}"))
        }
    }
}

fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    seconds.to_string()
}

pub fn offline_summary(notes: &str) -> String {
    let bullets = notes
        .lines()
        .filter(|line| line.trim_start().starts_with("- "))
        .take(6)
        .map(|line| line.trim().to_string())
        .collect::<Vec<_>>();

    if bullets.is_empty() {
        "暂无可总结的笔记。接入语音识别后，这里会生成会议纪要、关键决定和待办。".to_string()
    } else {
        format!(
            "本地摘要预览：\n{}\n\nAI 摘要接口接入后，可在这里生成正式会议纪要和待办。",
            bullets.join("\n")
        )
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
