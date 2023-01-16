# Check Da Helm
> Your helm releases are outdated, aren't they? Now you can check!

[![Version build](https://github.com/allanger/check-da-helm/actions/workflows/build-version.yaml/badge.svg)](https://github.com/allanger/check-da-helm/actions/workflows/build-version.yaml)
[![Version container](https://github.com/allanger/check-da-helm/actions/workflows/container-version.yaml/badge.svg)](https://github.com/allanger/check-da-helm/actions/workflows/container-version.yaml)
[![Stable container](https://github.com/allanger/check-da-helm/actions/workflows/container-stable.yaml/badge.svg)](https://github.com/allanger/check-da-helm/actions/workflows/container-stable.yaml)

## What's this?
It's a simple command line tool that lets you check whether your helm releases (currently installed by helmfile or argo) are outdated or not. Why it's created? But the main reason why it's created, is a necessity to check if helm releases that you have installed in your cluster still exist in repos. Once `Bitnami` removed old charts from their main repo and, I believe, everybody needed then some time to understand what happened. So I decided to write this tool. I was checking helmfiles and testing if chart were still in repos. And in case something is broken, I would be notified in the morning. Of course, broken helm charts are something you'll eventually know about, but it just feels better to know about them with this simple cli.

## Install 
### Dependencies
Depending on the tool you want to use `cdm` with, you must have either `helmfile` or `argocd` installed. And in any case you need to have `helm`

### Download 

Get executable from github releases

Prebuilt binaries exist for **Linux x86_64** and **MacOS arm64** and **x86_64**

Don't forget to add the binary to $PATH
```BASH
$ curl https://raw.githubusercontent.com/allanger/check-da-helm/main/scripts/download_cdh.sh | bash
$ cdm -h
```

### Build from source
1. Build binary
```BASH
$ cargo build --release
``` 
2. Run `gum help`

