use std::{collections::BTreeMap, fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

pub(crate) const MAX_DRAFT_CHARACTERS: usize = 200_000;
pub(crate) const MAX_PAGE_KEY_BYTES: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DraftPageKey(String);

impl DraftPageKey {
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        if raw.is_empty()
            || raw.len() > MAX_PAGE_KEY_BYTES
            || !raw.starts_with('/')
            || raw.starts_with("//")
            || !raw
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_' | b'-'))
        {
            return None;
        }

        Some(Self(raw.to_owned()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftFile {
    version: u8,
    #[serde(default)]
    drafts: BTreeMap<String, String>,
}

#[derive(Debug)]
pub(crate) struct DraftStore {
    path: PathBuf,
    drafts: BTreeMap<String, String>,
}

impl DraftStore {
    pub(crate) fn empty(path: PathBuf) -> Self {
        Self {
            path,
            drafts: BTreeMap::new(),
        }
    }

    pub(crate) fn load(path: PathBuf) -> io::Result<Self> {
        let drafts = match fs::read(&path) {
            Ok(bytes) => {
                let parsed: DraftFile = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
                if parsed.version != 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "unsupported draft-store version",
                    ));
                }
                parsed
                    .drafts
                    .into_iter()
                    .filter(|(key, text)| {
                        DraftPageKey::parse(key).is_some()
                            && !text.trim().is_empty()
                            && text.chars().count() <= MAX_DRAFT_CHARACTERS
                    })
                    .collect()
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(error),
        };

        Ok(Self { path, drafts })
    }

    pub(crate) fn draft(&self, key: &DraftPageKey) -> Option<&str> {
        self.drafts.get(key.as_str()).map(String::as_str)
    }

    pub(crate) fn save_draft(&mut self, key: &DraftPageKey, raw_text: &str) -> io::Result<bool> {
        let text = normalize_draft(raw_text);
        if text.trim().is_empty() {
            return Ok(false);
        }

        if self
            .drafts
            .get(key.as_str())
            .is_some_and(|saved| saved == &text)
        {
            return Ok(false);
        }

        self.drafts.insert(key.as_str().to_owned(), text);
        self.persist()?;
        Ok(true)
    }

    fn persist(&self) -> io::Result<()> {
        let parent = self.path.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "draft store has no parent directory",
            )
        })?;
        fs::create_dir_all(parent)?;

        let file = DraftFile {
            version: 1,
            drafts: self.drafts.clone(),
        };
        let bytes = serde_json::to_vec(&file).map_err(io::Error::other)?;
        let temp_path = self.path.with_extension("tmp");

        fs::write(&temp_path, bytes)?;
        fs::rename(&temp_path, &self.path)
    }
}

fn normalize_draft(raw_text: &str) -> String {
    let normalized = raw_text.replace("\r\n", "\n");
    normalized.chars().take(MAX_DRAFT_CHARACTERS).collect()
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

    fn test_path() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!(
                "mado-draft-store-test-{}-{nonce}-{}",
                std::process::id(),
                NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
            ))
            .join("drafts.json")
    }

    #[test]
    fn page_keys_are_path_scoped_and_bounded() {
        assert_eq!(
            DraftPageKey::parse("/c/1234-abcd").unwrap().as_str(),
            "/c/1234-abcd"
        );
        assert!(DraftPageKey::parse("/").is_some());
        assert!(DraftPageKey::parse("//evil").is_none());
        assert!(DraftPageKey::parse("/c/has.dot").is_none());
        assert!(DraftPageKey::parse("relative").is_none());
        assert!(DraftPageKey::parse(&format!("/{}", "a".repeat(MAX_PAGE_KEY_BYTES))).is_none());
    }

    #[test]
    fn store_round_trips_drafts_by_page_key() {
        let path = test_path();
        let page_a = DraftPageKey::parse("/c/a").unwrap();
        let page_b = DraftPageKey::parse("/c/b").unwrap();

        let mut store = DraftStore::load(path.clone()).unwrap();
        assert!(store.save_draft(&page_a, "draft a").unwrap());
        assert!(store.save_draft(&page_b, "draft b").unwrap());

        let loaded = DraftStore::load(path.clone()).unwrap();
        assert_eq!(loaded.draft(&page_a), Some("draft a"));
        assert_eq!(loaded.draft(&page_b), Some("draft b"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn empty_page_events_never_delete_existing_draft() {
        let path = test_path();
        let page = DraftPageKey::parse("/c/example").unwrap();

        let mut store = DraftStore::load(path.clone()).unwrap();
        store.save_draft(&page, "keep me").unwrap();
        assert!(!store.save_draft(&page, "   \n").unwrap());

        let loaded = DraftStore::load(path.clone()).unwrap();
        assert_eq!(loaded.draft(&page), Some("keep me"));

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn draft_length_is_bounded_and_line_endings_are_normalized() {
        let path = test_path();
        let page = DraftPageKey::parse("/").unwrap();
        let raw = format!("a\r\n{}", "x".repeat(MAX_DRAFT_CHARACTERS + 10));

        let mut store = DraftStore::load(path.clone()).unwrap();
        store.save_draft(&page, &raw).unwrap();

        let saved = store.draft(&page).unwrap();
        assert!(!saved.contains("\r\n"));
        assert_eq!(saved.chars().count(), MAX_DRAFT_CHARACTERS);

        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn identical_page_keys_remain_isolated_by_profile_store_path() {
        let root = test_path().parent().unwrap().to_path_buf();
        let path_a = root.join("profile-a/drafts.json");
        let path_b = root.join("profile-b/drafts.json");
        let page = DraftPageKey::parse("/c/same-page").unwrap();

        let mut a = DraftStore::load(path_a.clone()).unwrap();
        let mut b = DraftStore::load(path_b.clone()).unwrap();
        a.save_draft(&page, "profile a").unwrap();
        b.save_draft(&page, "profile b").unwrap();

        assert_eq!(
            DraftStore::load(path_a).unwrap().draft(&page),
            Some("profile a")
        );
        assert_eq!(
            DraftStore::load(path_b).unwrap().draft(&page),
            Some("profile b")
        );

        fs::remove_dir_all(root).unwrap();
    }
}
