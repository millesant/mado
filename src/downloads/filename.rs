use std::path::{Path, PathBuf};

const MAX_FILENAME_BYTES: usize = 180;

pub(crate) fn sanitize_filename(suggested: &str) -> String {
    let mut cleaned = String::with_capacity(suggested.len().min(MAX_FILENAME_BYTES));

    for character in suggested.chars() {
        let replacement = match character {
            '/' | '\\' | ':' => '-',
            character if character.is_control() => continue,
            character => character,
        };

        if cleaned.len() + replacement.len_utf8() > MAX_FILENAME_BYTES {
            break;
        }
        cleaned.push(replacement);
    }

    let cleaned = cleaned
        .trim()
        .trim_start_matches(['.', '-', '_'])
        .trim_end_matches('.')
        .trim()
        .to_owned();

    if cleaned.is_empty() {
        "chatgpt-download".to_owned()
    } else {
        cleaned
    }
}

pub(crate) fn safe_unique_destination(
    directory: &Path,
    suggested: &str,
    reserved: impl Fn(&Path) -> bool,
) -> PathBuf {
    let sanitized = sanitize_filename(suggested);
    let path = Path::new(&sanitized);
    let extension = path.extension().and_then(|value| value.to_str());
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("chatgpt-download");

    let mut candidate = directory.join(&sanitized);
    let mut index = 1_u32;

    while candidate.exists() || reserved(&candidate) {
        let name = match extension {
            Some(extension) if !extension.is_empty() => {
                format!("{stem}-{index}.{extension}")
            }
            _ => format!("{stem}-{index}"),
        };
        candidate = directory.join(name);
        index = index.saturating_add(1);
    }

    candidate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_path_separators_control_chars_and_empty_names() {
        assert_eq!(sanitize_filename("../../etc/passwd"), "etc-passwd");
        assert_eq!(sanitize_filename("..\\..\\secret.txt"), "secret.txt");
        assert_eq!(sanitize_filename(" report:\n.txt "), "report-.txt");
        assert_eq!(sanitize_filename("..."), "chatgpt-download");
        assert_eq!(sanitize_filename("   "), "chatgpt-download");
    }

    #[test]
    fn limits_filename_utf8_bytes() {
        assert!(
            sanitize_filename(&"a".repeat(MAX_FILENAME_BYTES + 50)).len() <= MAX_FILENAME_BYTES
        );
        assert!(sanitize_filename(&"😀".repeat(MAX_FILENAME_BYTES)).len() <= MAX_FILENAME_BYTES);
    }

    #[test]
    fn generated_destination_never_escapes_directory() {
        let directory = std::env::temp_dir().join("mado-download-filename-test");
        let candidate = safe_unique_destination(&directory, "../../outside.txt", |_| false);

        assert_eq!(candidate.parent(), Some(directory.as_path()));
        assert_eq!(
            candidate.file_name().and_then(|value| value.to_str()),
            Some("outside.txt")
        );
    }

    #[test]
    fn reserved_destination_gets_unique_suffix() {
        let directory = PathBuf::from("/tmp/mado-downloads");
        let reserved = directory.join("report.txt");
        let candidate = safe_unique_destination(&directory, "report.txt", |path| path == reserved);

        assert_eq!(candidate, directory.join("report-1.txt"));
    }
}
