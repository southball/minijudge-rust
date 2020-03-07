#!/bin/bash

sudo RUST_BACKTRACE=full $HOME/.cargo/bin/cargo run -- \
  --sandboxes 4 \
  --checker ./test/checker.cpp \
  --language python3 \
  --metadata ./test/metadata.py.yml \
  --source ./test/source.py \
  --testcases ./test/testcases \
  --testlib ./test/testlib.h \
  --verdict-format yaml \
  --verdict ./verdict.py.txt \
  $* \
  1> >(sed $'s,.*,\e[1;33m&\e[m,'>&2);