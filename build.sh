#!/bin/sh

mkdir -p build
cd build

echo ">> Compiling libpaws-tests"
rustc -g --test ../src/lib/paws.rs -o libpaws-tests || exit 1

echo ">> Running libpaws-tests"
if ! ./libpaws-tests; then
  echo "E: Aborting compilation due to failed tests" > /dev/stderr
  exit 1
fi

echo ">> Compiling libpaws"
rustc -O ../src/lib/paws.rs || exit 1

echo ">> Compiling paws_rs"
rustc -O -L . ../src/bin/paws_rs.rs || exit 1

echo ">> Generating documentation"
rustdoc ../src/lib/paws.rs -o ../doc
