#!/bin/bash
case "$(uname)" in

"Darwin")
  SYSTEM="apple-darwin"
  case $(uname -m) in
  "arm64")
    TARGET="aarch64-$SYSTEM"
    ;;
  "x86_64")
    TARGET="x86_64-$SYSTEM"
    ;;
  *)
    echo "Unsuported target"
    exit 1
    ;;
  esac
  ;;
"Linux")
  SYSTEM="unknown-linux-gnu"
  case $(uname -m) in
  "x86_64")
    TARGET="x86_64-$SYSTEM"
    ;;
  *)
    echo "Unsuported target"
    exit 1
    ;;
  esac
  ;;
*)
  echo "Signal number $1 is not processed"
  exit 1
  ;;
esac
LATEST_VERSION="v$(curl -s https://raw.githubusercontent.com/allanger/check-da-helm/main/Cargo.toml | awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2); exit}')"
echo "Downloading $LATEST_VERSION"

RELEASE_NAME=cdh-$LATEST_VERSION-$TARGET
RELEASE_URL="https://github.com/allanger/check-da-helm/releases/download/$LATEST_VERSION/$RELEASE_NAME"
echo "Link for downloading: $RELEASE_URL"
curl -LJO $RELEASE_URL

mv $RELEASE_NAME cdh
chmod +x cdh

echo 'Make sure that cdh is in your $PATH'
echo 'Try: '
echo ' $ export PATH=$PATH:$PWD'
echo ' $ cdh -h'
