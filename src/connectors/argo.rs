use clap::Arg;
use log::{debug, info, error};
use serde_json::from_str;
use crate::types::{self, HelmRepo};

use super::Connector;
use std::{borrow::Borrow, io::{Result, Error, ErrorKind}, process::Command};

pub(crate) struct Argo;

impl Connector for Argo {
    type ConnectorType = Argo;

    fn get_app(&self) -> Result<Vec<types::HelmChart>> {
        let cmd: String = "argocd app list -o json | jq '[.[] | {chart: .spec.source.chart, version: .spec.source.targetRevision}]'".to_string();

        debug!("executing '${}'", cmd);
        let output = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("argo is failed");
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
        info!("syncing helm repos");
        let cmd: String = "argocd app list -o json | jq '[ .[] | {name: .spec.source.chart, url: .spec.source.repoURL} ]'".to_string();
        let output = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("helmfile is failed");
        info!("{:?}", output.clone());
        if output.status.success() {
            let repos: Vec<HelmRepo> = serde_json::from_slice(&output.stdout).unwrap();
            info!("adding repositories");
            for repo in repos.iter() {
                let name = repo.name.clone();
                if name.is_some() {
                    info!(
                        "syncing {} with the origin {}",
                        name.clone().unwrap(),
                        repo.url
                    );
                    let cmd = format!(
                        "helm repo add {} {}",
                        name.clone().unwrap(),
                        repo.url.clone()
                    );
                    debug!("running {}", cmd);
                    let output = Command::new("bash")
                        .arg("-c")
                        .arg(cmd)
                        .output()
                        .expect("helm repo sync is failed");
                    match output.status.success() {
                        true => {
                            info!(
                                "{} with the origin {} is synced successfully",
                                name.unwrap(),
                                repo.url
                            );
                        }
                        false => {
                            error!(
                                "{} with the origin {} can't be synced",
                                name.unwrap(),
                                repo.url
                            )
                        }
                    }
                }
            }
            let cmd = "helm repo update";
            let output = Command::new("bash")
                .arg("-c")
                .arg(cmd)
                .output()
                .expect("helm repo sync is failed");
            match output.status.success() {
                true => {
                    info!("repositories are updated successfully");
                }
                false => {
                    error!(
                        "repositories can't be updated, {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
            }
    
            Ok(())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            ))
        }
    
    }

}
impl Argo{
pub(crate) fn init() -> Argo {
    Argo
}
}