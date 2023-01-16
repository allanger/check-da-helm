#!/bin/bash
echo 'renaming cdh to cdh-$VERSION-$SYSTEM format'
mkdir -p release
echo "version - $CDH_VERSION"
for BUILD in build*; do
  SYSTEM=$(echo $BUILD | sed -e 's/build-//g')
  echo "system - $SYSTEM"
  cp $BUILD/cdh release/cdh-$CDH_VERSION-$SYSTEM
done
ls release
