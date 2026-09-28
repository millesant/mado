pub mod model;
pub mod storage;

use std::{env, fmt};

pub(crate) use model::ProfileId;

#[derive(Debug)]
pub(crate) enum ProfileSelectionError {
    InvalidId(String),
    NonUnicode,
}

impl fmt::Display for ProfileSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId(id) => write!(f, "invalid MADO_PROFILE value: {id:?}"),
            Self::NonUnicode => write!(f, "MADO_PROFILE is not valid UTF-8"),
        }
    }
}

pub(crate) fn selected_profile() -> Result<ProfileId, ProfileSelectionError> {
    match env::var("MADO_PROFILE") {
        Ok(value) => ProfileId::parse(&value).map_err(|_| ProfileSelectionError::InvalidId(value)),
        Err(env::VarError::NotPresent) => Ok(ProfileId::default()),
        Err(env::VarError::NotUnicode(_)) => Err(ProfileSelectionError::NonUnicode),
    }
}
