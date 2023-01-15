mod connectors;
mod types;

use clap::{Parser, ValueEnum};
use connectors::{Argo, Connector, Helm, Helmfile};
use handlebars::Handlebars;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{
    borrow::Borrow,
    fmt::{self, format},
    io::{Error, ErrorKind, Result},
    process::{exit, Command},
};
use tabled::Tabled;
use version_compare::{Cmp, Version};

use crate::types::HelmChart;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Kinds {
    Argo,
    Helm,
    Helmfile,
}

/// Check you helm releaseas managed by Argo
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Type of the
    #[clap(long, value_enum)]
    kind: Kinds,
    /// Path to the helmfile
    #[clap(short, long, value_parser, default_value = "./")]
    path: String,
    /// Should execution be failed if you have outdated charts
    #[clap(short, long, action, default_value_t = false, env = "OUTDATED_FAIL")]
    outdated_fail: bool,
    /// Set to true if you don't want to sync repositories
    #[clap(short, long, action, default_value_t = false)]
    no_sync: bool,
}

/// A struct to write helm repo description to
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Repo {
    name: Option<String>,
    url: String,
}

/// Struct for parsing charts info from helmfile
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct LocalCharts {
    #[serde(alias = "name", alias = "chart")]
    chart: Option<String>,
    version: Option<String>,
}

/// Three possible statuses of versions comparison
#[derive(Clone, Serialize)]
enum Status {
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
#[derive(Clone, Tabled, Serialize)]
struct ExecResult {
    name: String,
    latest_version: String,
    current_version: String,
    status: Status,
}

// Implementation for the ExecResult struct
impl ExecResult {
    fn new(name: String, latest_version: String, current_version: String, status: Status) -> Self {
        Self {
            name,
            latest_version,
            current_version,
            status,
        }
    }
}

fn main() {
    // Preparations step
    env_logger::init();
    let args = Args::parse();
    let mut result: Vec<ExecResult> = Vec::new();

    let charts = match args.kind {
        Kinds::Argo => Argo::init().get_app(),
        Kinds::Helm => Helm::init().get_app(),
        Kinds::Helmfile => Helmfile::init(args.path.clone()).get_app(),
    }
    .unwrap();

    if !args.no_sync {
        info!("syncing helm repositories");
        let res = match  args.kind {
            Kinds::Argo => Argo::init().sync_repos(),
            Kinds::Helm => Helm::init().sync_repos(),
            Kinds::Helmfile => Helmfile::init(args.path).sync_repos(),
        };
        match res {
            Ok(_) => info!("helm repos are synced"),
            Err(err) => error!("couldn't sync repos', {}", err),
        }
    }

    charts.iter().for_each(|a| {
        let err = check_chart(&mut result, a);
    });

    // Parse the helmfile
    // Handling the result
    match handle_result(&result, args.outdated_fail) {
        Ok(result) => {
            if result {
                exit(1);
            }
        }
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    };
}

fn check_chart(result: &mut Vec<ExecResult>, local_chart: &types::HelmChart) -> Result<()> {
    if local_chart.clone().name.is_some() {
        let version = local_chart.version.clone().unwrap();
        let chart = local_chart.name.clone().unwrap();
        return match version.is_empty() {
            true => {
                warn!(
                    "version is not specified for the '{}' chart, skipping",
                    chart
                );
                Ok(())
            }
            false => {
                info!("checking {} - {}", chart, version);
                let cmd = format!(
                    "helm search repo {}/{} --versions --output json",
                    chart, chart
                );
                debug!("executing '${}'", cmd);
                let output = Command::new("bash")
                    .arg("-c")
                    .arg(cmd)
                    .output()
                    .expect("helmfile is failed");
                let helm_stdout = String::from_utf8_lossy(&output.stdout);

                // Remove "v" from version definitions
                let mut versions: Vec<HelmChart> = from_str(helm_stdout.borrow()).unwrap();
                versions.iter_mut().for_each(|f| {
                    if f.version.is_some() {
                        f.version = Some(f.version.as_ref().unwrap().replace('v', ""));
                    }
                });
                // Create a Version from the chart version string
                let local = Version::from(&version).unwrap();
                let mut current_version: String = "0.0.0".to_string();

                // Get the latest remote version
                for v in versions.iter() {
                    current_version = get_newer_version(
                        current_version.clone(),
                        v.version.as_ref().unwrap().clone(),
                    );
                }
                let remote = Version::from(current_version.as_str()).unwrap();
                let status: Status = if versions.contains(&HelmChart {
                    name: Some(format!("{}/{}", chart.clone(), chart.clone())),
                    version: Some(version.clone()),
                }) {
                    match local.compare(remote.clone()) {
                        Cmp::Lt => Status::Outdated,
                        Cmp::Eq => Status::Uptodate,
                        Cmp::Gt => Status::Missing,
                        _ => unreachable!(),
                    }
                } else {
                    Status::Missing
                };

                result.push(ExecResult::new(
                    chart.clone(),
                    current_version.clone(),
                    version.clone(),
                    status,
                ));

                Ok(())
            }
        };
    } else {
        return Ok(());
    }
}

/// Handle result
fn handle_result(result: &Vec<ExecResult>, outdated_fail: bool) -> Result<bool> {
    let mut failed = false;
    for r in result.clone() {
        match r.status {
            Status::Uptodate => info!("{} is up-to-date", r.name),
            Status::Outdated => {
                if outdated_fail {
                    failed = true
                }
                warn!(
                    "{} is outdated. Current version is {}, but the latest is {}",
                    r.name, r.current_version, r.latest_version
                );
            }
            Status::Missing => {
                failed = true;
                error!(
                    "{} is broken. Current version is {}, but it can't be found in the repo",
                    r.name, r.current_version
                );
            }
        }
    }
    let template = r#"
<table>
    <tr>
        <th>Chart Name</th>
        <th>Current Version</th>
        <th>Latest Version</th>
        <th>Status</th>
    </tr>
    {{#each this as |tr|}}
    <tr>
        <th>{{tr.name}}</th>
        <th>{{tr.current_version}}</th>
        <th>{{tr.latest_version}}</th>
        <th>{{tr.status}}</th>
    </tr>
    {{/each}}
</table>
"#;
    let mut reg = Handlebars::new();

    // TODO: Handle this error
    reg.register_template_string("html_table", template)
        .unwrap();

    match reg.render("html_table", &result) {
        Ok(res) => println!("{}", res),
        Err(err) => error!("{}", err),
    };
    Ok(failed)
}

/// Downloading repos from repositories
fn repo_sync() -> Result<()> {
    info!("syncing helm repos");
    let cmd: String = "argocd app list -o json | jq '[ .[] | {name: .spec.source.chart, url: .spec.source.repoURL} ]'".to_string();
    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("helmfile is failed");
    info!("{:?}", output.clone());
    if output.status.success() {
        let repos: Vec<Repo> = serde_json::from_slice(&output.stdout).unwrap();
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

/// Run helmfile list and write the result into struct
fn parse_argo_apps() -> Result<Vec<LocalCharts>> {
    let cmd: String = "argocd app list -o json | jq '[.[] | {chart: .spec.source.chart, version: .spec.source.targetRevision}]'".to_string();

    debug!("executing '${}'", cmd);
    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("helmfile is failed");
    let helm_stdout = String::from_utf8_lossy(&output.stdout);

    match from_str::<Vec<LocalCharts>>(Borrow::borrow(&helm_stdout)) {
        Ok(mut charts) => {
            charts.dedup();
            Ok(charts)
        }
        Err(err) => Err(err.into()),
    }
}

/// Takes two version and returns the newer one.
fn get_newer_version(v1: String, v2: String) -> String {
    match Version::from(&v1.replace('v', ""))
        .unwrap()
        .compare(Version::from(&v2.replace('v', "")).unwrap().clone())
    {
        Cmp::Eq => v1,
        Cmp::Lt => v2,
        Cmp::Gt => v1,
        _ => unreachable!(),
    }
}
