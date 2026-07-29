use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use crate::sessions::{SessionDetails, SessionStore};

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub score: usize,
    pub reasons: Vec<String>,
    pub snippet: String,
    pub highlighted_snippet: String,
    pub match_text: String,
    pub updated_at: u64,
    pub session: SessionDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStats {
    pub sessions: usize,
    pub documents: usize,
    pub terms: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedIndex {
    stats: IndexStats,
    updated_at: u64,
    documents: Vec<IndexedSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexedSession {
    slug: String,
    title: String,
    updated_at: u64,
    searchable_text: String,
    fields: BTreeMap<String, BTreeMap<String, usize>>,
}

pub fn rebuild(store: &SessionStore) -> Result<IndexStats, String> {
    fs::create_dir_all("../ai-listen-data").map_err(|error| error.to_string())?;
    let sessions = store.list_sessions()?;
    let mut terms = BTreeSet::new();
    let mut documents = Vec::new();

    for session in &sessions {
        let fields = session_fields(session);
        for token in fields.values().flat_map(|counts| counts.keys()) {
            terms.insert(token.clone());
        }
        documents.push(IndexedSession {
            slug: session.slug.clone(),
            title: session.title.clone(),
            updated_at: session_updated_at(session),
            searchable_text: searchable_text(session),
            fields,
        });
    }

    let stats = IndexStats {
        sessions: sessions.len(),
        documents: documents.len(),
        terms: terms.len(),
    };
    let index = PersistedIndex {
        stats: stats.clone(),
        updated_at: now_secs(),
        documents,
    };

    fs::write(
        index_path(),
        serde_json::to_string_pretty(&index).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(stats)
}

pub fn search(store: &SessionStore, query: &str) -> Result<Vec<SearchHit>, String> {
    let query_terms = tokenize(query);
    if query_terms.is_empty() {
        return Ok(store
            .list_sessions()?
            .into_iter()
            .map(|session| SearchHit {
                score: 0,
                reasons: vec!["全部会议".to_string()],
                snippet: String::new(),
                highlighted_snippet: String::new(),
                match_text: String::new(),
                updated_at: session_updated_at(&session),
                session,
            })
            .collect());
    }

    let index = load_or_rebuild(store)?;
    let mut hits = Vec::new();
    for document in index.documents {
        let mut score = 0;
        let mut reasons = Vec::new();

        for term in &query_terms {
            for (field, tokens) in &document.fields {
                let count = tokens.get(term).copied().unwrap_or(0);
                if count > 0 {
                    let weight = match field.as_str() {
                        "标题" => 5,
                        "笔记" => 3,
                        "转写" => 3,
                        _ => 1,
                    };
                    score += count * weight;
                    reasons.push(format!("{field}: {term}"));
                }
            }
        }

        if score > 0 {
            reasons.sort();
            reasons.dedup();
            let session = store.read_session(&document.slug)?;
            let snippet = make_snippet(&document.searchable_text, &query_terms);
            let highlighted_snippet = highlight_snippet(&snippet, &query_terms);
            let match_text = first_match_text(&document.searchable_text, &query_terms);
            hits.push(SearchHit {
                score,
                reasons,
                snippet,
                highlighted_snippet,
                match_text,
                updated_at: document.updated_at,
                session,
            });
        }
    }

    hits.sort_by(|left, right| right.score.cmp(&left.score));
    Ok(hits)
}

fn load_or_rebuild(store: &SessionStore) -> Result<PersistedIndex, String> {
    let path = index_path();
    if !path.exists() {
        rebuild(store)?;
    }

    let text = fs::read_to_string(&path).map_err(|error| error.to_string())?;
    serde_json::from_str(&text).map_err(|error| {
        let _ = fs::remove_file(&path);
        format!("failed to read index; rebuild it and try again: {error}")
    })
}

fn session_fields(session: &SessionDetails) -> BTreeMap<String, BTreeMap<String, usize>> {
    BTreeMap::from([
        ("标题".to_string(), count_terms(&session.title)),
        ("笔记".to_string(), count_terms(&session.notes)),
        ("截图".to_string(), count_many(&session.screenshots)),
        ("录屏".to_string(), count_many(&session.recordings)),
        ("音频".to_string(), count_many(&session.audio)),
        ("转写".to_string(), count_terms(&transcript_text(session))),
    ])
}

fn searchable_text(session: &SessionDetails) -> String {
    [
        session.title.as_str(),
        session.notes.as_str(),
        &session.screenshots.join("\n"),
        &session.recordings.join("\n"),
        &session.audio.join("\n"),
        &transcript_text(session),
    ]
    .join("\n")
}

fn transcript_text(session: &SessionDetails) -> String {
    session
        .transcripts
        .iter()
        .map(|path| fs::read_to_string(path).unwrap_or_else(|_| path.to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

fn make_snippet(text: &str, terms: &[String]) -> String {
    let lower = text.to_lowercase();
    let hit = terms
        .iter()
        .filter_map(|term| lower.find(term).map(|index| (index, term)))
        .min_by_key(|(index, _)| *index);

    let Some((index, _)) = hit else {
        return text.chars().take(160).collect::<String>();
    };

    let start = lower[..index]
        .char_indices()
        .rev()
        .nth(40)
        .map(|(i, _)| i)
        .unwrap_or(0);
    let end = lower[index..]
        .char_indices()
        .nth(120)
        .map(|(i, _)| index + i)
        .unwrap_or_else(|| text.len());

    let mut snippet = String::new();
    if start > 0 {
        snippet.push_str("...");
    }
    snippet.push_str(text[start..end].trim());
    if end < text.len() {
        snippet.push_str("...");
    }
    snippet
}

fn first_match_text(text: &str, terms: &[String]) -> String {
    let lower = text.to_lowercase();
    terms
        .iter()
        .filter_map(|term| lower.find(term).map(|index| (index, term)))
        .min_by_key(|(index, _)| *index)
        .map(|(_, term)| term.to_string())
        .unwrap_or_default()
}

fn highlight_snippet(snippet: &str, terms: &[String]) -> String {
    let mut highlighted = snippet.to_string();
    for term in terms {
        highlighted = highlighted.replace(term, &format!("<mark>{term}</mark>"));
    }
    highlighted
}

fn session_updated_at(session: &SessionDetails) -> u64 {
    std::iter::once(session.path.as_str())
        .chain(session.screenshots.iter().map(String::as_str))
        .chain(session.recordings.iter().map(String::as_str))
        .chain(session.audio.iter().map(String::as_str))
        .chain(session.transcripts.iter().map(String::as_str))
        .filter_map(|path| fs::metadata(path).ok()?.modified().ok())
        .filter_map(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .max()
        .unwrap_or_else(now_secs)
}

fn count_many(items: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for item in items {
        for token in tokenize(item) {
            *counts.entry(token).or_insert(0) += 1;
        }
    }
    counts
}

fn count_terms(text: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for token in tokenize(text) {
        *counts.entry(token).or_insert(0) += 1;
    }
    counts
}

fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            current.push(ch);
        } else if !current.is_empty() {
            tokens.push(std::mem::take(&mut current));
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn index_path() -> PathBuf {
    PathBuf::from("../ai-listen-data").join(".index.json")
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
