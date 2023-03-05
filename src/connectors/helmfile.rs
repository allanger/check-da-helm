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
            "helmfile -f {} -e {} list --output json | jq '[.[] | {{chart: .chart, version: .version, name: .name}}]'",
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

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use std::io::Write;
    use crate::connectors::{Helmfile, Connector};
    use crate::types;

    static HELMFILE_EXAMPLE: &str = "repositories:\n
  - name: argo\n
    url: https://argoproj.github.io/argo-helm\n
releases:\n
  - name: argocd\n
    installed: true\n
    namespace: argocd\n
    createNamespace: true\n
    chart: argo/argo-cd\n
    version: 5.23.3\n
    values:\n
      - server:\n
          extraArgs:\n
            - --insecure";

    #[test]
    fn test_helmfile() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}", HELMFILE_EXAMPLE.clone()).unwrap();
        let path = file.into_temp_path();
        let helmfile_app = Helmfile::init(path.to_string_lossy().to_string(), "default".to_string()).get_app().unwrap();
        let app = types::HelmChart{
            chart: Some("argo/argo-cd".to_string()),
            name: Some("argocd".to_string()),
            version: Some("5.23.3".to_string()),
        };
        let apps: Vec<types::HelmChart> = vec![app];
        assert_eq!(apps, helmfile_app);
        path.close().unwrap();
    }
}
