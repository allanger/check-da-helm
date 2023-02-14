#!/bin/sh
case $(uname -m) in
  "arm64"|"aarch64")
    PLATFORM="arm64"
    ;;
  "x86_64")
    PLATFORM="amd64"
    ;;
  *)
    echo "Unsuported target"
    exit 1
    ;;
esac
echo "Downloading $2 from $1"
curl -LJO $1 $2
