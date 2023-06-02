mod connectors;
mod output;
mod types;
use clap::{arg, command, Parser, Subcommand, ValueEnum};
use connectors::{Argo, Connector, Helm, Helmfile};
use log::{debug, error, info, warn};
use output::Output;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{
    borrow::Borrow,
    io::Result,
    process::{exit, Command},
};
use types::ExecResult;
use version_compare::{Cmp, Version};

use crate::types::{HelmChart, Status};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Kinds {
    Argo,
    //Helm,
    Helmfile,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Outputs {
    Yaml,
    HTML,
}

/// Check you helm releaseas managed by Argo
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// How do you install your helm charts
    #[clap(long, value_enum)]
    kind: Kinds,
    /// What kind of output would you like to receive?
    #[clap(long, value_enum, default_value = "yaml")]
    output: Outputs,
    /// Path to the helmfile
    #[clap(short, long, value_parser, default_value = "./")]
    path: String,
    /// Pass an environment to the helmfile
    #[arg(long, required = false, default_value = "default")]
    helmfile_environment: String,
    /// Should execution be failed if you have outdated charts
    #[clap(short, long, action, default_value_t = false, env = "OUTDATED_FAIL")]
    outdated_fail: bool,
    /// Set to true if you don't want to sync repositories
    #[clap(short, long, action, default_value_t = false)]
    no_sync: bool,
    /// Set to true if you don't wante to see up-to-date charts in the output
    #[clap(long, action, default_value_t = true)]
    no_uptodate: bool
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Generate {
        #[arg(value_name = "SHELL", default_missing_value = "zsh")]
        shell: clap_complete::shells::Shell,
    },
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

// Implementation for the ExecResult struct

fn main() {
    // Preparations step
    env_logger::init();
    let args = Args::parse();
    let mut result: Vec<ExecResult> = Vec::new();
    
    if !args.no_sync {
        info!("syncing helm repositories");
        let res = match args.kind {
            Kinds::Argo => Argo::init().sync_repos(),
            // Kinds::Helm => Helm::init().sync_repos(),
            Kinds::Helmfile => Helmfile::init(args.path.clone(), args.helmfile_environment.clone()).sync_repos(),
        };
        match res {
            Ok(_) => info!("helm repos are synced"),
            Err(err) => error!("couldn't sync repos', {}", err),
        }
    }

    let charts = match args.kind {
        Kinds::Argo => Argo::init().get_app(),
        //Kinds::Helm => Helm::init().get_app(),
        Kinds::Helmfile => Helmfile::init(args.path.clone(), args.helmfile_environment.clone()).get_app(),
    }
    .unwrap();

    charts.iter().for_each(|a| {
        debug!("{:?}", a);
        check_chart(&mut result, a).unwrap();
    });

    // Parse the helmfile
    // Handling the result
    match handle_result(&mut result, args.outdated_fail, args.output, args.no_uptodate) {
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
    if local_chart.chart.is_some() {
        let version = local_chart.version.clone().unwrap();
        let chart = local_chart.chart.clone().unwrap();
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
                    "helm search repo {} --versions --output json",
                    chart,
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
                    f.name = local_chart.name.clone();
                    f.chart = local_chart.chart.clone();
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
                debug!("{:?}", versions);
                let status: Status = if versions.contains(&HelmChart {
                    chart: Some(chart.clone()),
                    name: local_chart.name.clone(),
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
                    local_chart.name.clone().unwrap(),
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
fn handle_result(
    result: &mut Vec<ExecResult>,
    outdated_fail: bool,
    output_kind: Outputs,
    no_uptodate: bool,
) -> Result<bool> {
    let mut failed = false;
    if no_uptodate {
        result.retain(|r| r.status != types::Status::Uptodate)
    }
    for r in result.clone() {
        match r.status {
            Status::Uptodate => info!("{} is up-to-date", r.chart),
            Status::Outdated => {
                if outdated_fail {
                    failed = true
                }
                warn!(
                    "{} is outdated. Current version is {}, but the latest is {}",
                    r.chart, r.current_version, r.latest_version
                );
            }
            Status::Missing => {
                failed = true;
                error!(
                    "{} is broken. Current version is {}, but it can't be found in the repo",
                    r.chart, r.current_version
                );
            }
        }
    }

    match output_kind {
        Outputs::Yaml => print!("{}", output::YAML::print(result)?),
        Outputs::HTML => print!("{}", output::HTML::print(result)?),
    };

    Ok(failed)
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
