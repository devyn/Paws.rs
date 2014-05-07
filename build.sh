#!/bin/sh

mkdir -p build
cd build

echo ">> Compiling libpaws-tests"
rustc --test ../src/lib/paws.rs -o libpaws-tests || exit 1

echo ">> Running libpaws-tests"
if ! ./libpaws-tests; then
  echo "E: Aborting compilation due to failed tests" > /dev/stderr
  exit 1
fi

echo ">> Compiling libpaws"
rustc ../src/lib/paws.rs || exit 1

echo ">> Compiling paws_rs"
rustc -L . ../src/bin/paws_rs.rs || exit 1
