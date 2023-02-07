use log::debug;
use serde_json::from_str;

use crate::types;

use super::Connector;
use std::{borrow::Borrow, io::Result, process::Command};

pub(crate) struct Helmfile {
    path: String,
    env: String,
}

impl Connector for Helmfile {
    fn get_app(&self) -> Result<Vec<types::HelmChart>> {
        let cmd: String = format!(
            "helmfile -f {} -e {} list --output json | jq '[.[] | {{chart: .name, version: .version}}]'",
            self.path,
            self.env
        )
        .to_string();

        debug!("executing '${}'", cmd);
        let output = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("helmfile list is failed");
        let helm_stdout = String::from_utf8_lossy(&output.stdout);

        match from_str::<Vec<types::HelmChart>>(Borrow::borrow(&helm_stdout)) {
            Ok(mut charts) => {
                charts.dedup();
                Ok(charts)
            }
            Err(err) => Err(err.into()),
        }
    }
    fn sync_repos(&self) -> Result<()> {
        let cmd: String = format!("helmfile -f {} -e {} sync", self.path, self.env);
        Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("helmfile sync is failed");
        Ok(())
    }

    type ConnectorType = Helmfile;
}
impl Helmfile {
    pub(crate) fn init(path: String, env: String) -> Self {
        Self {path, env}
    }
}
