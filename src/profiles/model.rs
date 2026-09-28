use std::fmt;

pub(crate) const DEFAULT_PROFILE_ID: &str = "default";
const MAX_PROFILE_ID_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProfileId(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct InvalidProfileId;

impl ProfileId {
    pub(crate) fn parse(value: &str) -> Result<Self, InvalidProfileId> {
        let bytes = value.as_bytes();
        if bytes.is_empty()
            || bytes.len() > MAX_PROFILE_ID_LEN
            || !(bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
            || !bytes.iter().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
            })
        {
            return Err(InvalidProfileId);
        }

        Ok(Self(value.to_owned()))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ProfileId {
    fn default() -> Self {
        Self(DEFAULT_PROFILE_ID.to_owned())
    }
}

impl fmt::Display for ProfileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_ids_are_path_safe_and_canonical() {
        for valid in ["default", "work", "profile-2", "qa_test", "2-work"] {
            assert_eq!(ProfileId::parse(valid).unwrap().as_str(), valid);
        }

        for invalid in [
            "",
            "../work",
            "work/profile",
            ".hidden",
            "Work",
            "équipe",
            "space name",
            "a.thing",
        ] {
            assert!(ProfileId::parse(invalid).is_err(), "{invalid:?}");
        }

        assert!(ProfileId::parse(&"a".repeat(MAX_PROFILE_ID_LEN)).is_ok());
        assert!(ProfileId::parse(&"a".repeat(MAX_PROFILE_ID_LEN + 1)).is_err());
    }
}
