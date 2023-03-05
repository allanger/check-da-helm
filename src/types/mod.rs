use serde::{Deserialize, Serialize};
use std::fmt;
use tabled::Tabled;

/// Struct for parsing charts info from helmfile
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct HelmChart {
    // #[serde(alias = "name", alias = "chart")]
    pub(crate) name: Option<String>,
    // #[serde(alias = "name", alias = "chart")]
    pub(crate) chart: Option<String>,
    pub(crate) version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub(crate) struct HelmRepo {
    pub(crate) name: Option<String>,
    pub(crate) url: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Tabled, Serialize, Deserialize)]
pub(crate) struct ExecResult {
    pub(crate) name: String,
    pub(crate) chart: String,
    pub(crate) latest_version: String,
    pub(crate) current_version: String,
    pub(crate) status: Status,
}

impl ExecResult {
    pub(crate) fn new(
        name: String,
        chart: String,
        latest_version: String,
        current_version: String,
        status: Status,
    ) -> Self {
        Self {
            name,
            chart,
            latest_version,
            current_version,
            status,
        }
    }
}
