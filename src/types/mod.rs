use serde::{Deserialize, Serialize};
use std::fmt;

/// Struct for parsing charts info from helmfile
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct HelmChart {
    #[serde(alias = "name", alias = "chart")]
    pub(crate) name: Option<String>,
    pub(crate) version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct HelmRepo {
    pub(crate) name: Option<String>,
    pub(crate) url: String,
}

#[derive(Clone, Serialize)]
pub(crate) enum Status {
    Uptodate,
    Outdated,
    Missing,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Status::Uptodate => write!(f, "Up-to-date"),
            Status::Outdated => write!(f, "Outdated"),
            Status::Missing => write!(f, "Missing"),
        }
    }
}
