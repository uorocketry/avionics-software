#!/bin/bash
# Tool to prefix all python imports in a given proto path with a given package prefix.
# Since protobuf currently doesn't support option python_package, this is a workaround to ensure that generated python files are properly namespaced.
set -e

PROTO_PATH=$1
PREFIX=$2

if [ -z "$PROTO_PATH" ]; then
  echo "Usage: $0 <path> <package-prefix>";
  exit 1;
fi

if [ -z "$PREFIX" ]; then
  echo "Usage: $0 <path> <package-prefix>";
  exit 1;
fi

cd "$PROTO_PATH"

for dir in $(find . -type d -maxdepth 1 ! -path "."); do
  pkgName="$( echo $dir | perl -pE 's/^\.\///g' | perl -pE 's/\//\./g' )"
  echo "$pkgName -> $PREFIX.$pkgName"
  pkgRegex="$( echo $pkgName | perl -pE 's/\./\\\./g' )"
  find $dir -type f -name "*.py*" | xargs perl -pi -E "s/^from ($pkgName)/from $PREFIX.\$1/g"
  find $dir -type f -name "*.py*" | xargs perl -pi -E "s/^import ($pkgName)/import $PREFIX.\$1/g"
  find $dir -type f -name "*.pyi" | xargs perl -pi -E "s/\[($pkgName)\./\[$PREFIX.\$1\./g"
done