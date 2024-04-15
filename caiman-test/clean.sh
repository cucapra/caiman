#! /bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
pushd $SCRIPT_DIR > /dev/null
if [ "$1" = "-a" ]; then 
	cargo clean
fi
find . -path "./src/*.rs" -and -not -name "util.rs" -delete
echo "" > ./src/lib.rs
popd > /dev/null
