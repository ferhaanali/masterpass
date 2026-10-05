use std::{
    ffi::OsStr,
    path::{Path},
    fmt,
};
use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize, Default, Debug, Clone, Hash, PartialEq, Eq)]
pub struct StringPath(pub String);

impl StringPath {
    pub fn from_str(path: &str) -> Self {
        Self(path.to_string())
    }

    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn join(&self, component: impl AsRef<str>) -> Self {
        let component = component.as_ref();
        if self.0.is_empty() {
            Self(component.to_string())
        } else {
            Self(format!("{}/{}", self.0, component))
        }
    }

    pub fn parent(&self) -> Option<Self> {
        Path::new(&self.0)
            .parent()
            .map(|p| Self(p.to_string_lossy().into_owned()))
    }
}

impl AsRef<Path> for StringPath {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl AsRef<OsStr> for StringPath {
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.0)
    }
}

impl fmt::Display for StringPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) ->fmt::Result {
        write!(f, "{}", self.0)
    }
}


