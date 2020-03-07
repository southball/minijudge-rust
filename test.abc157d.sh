#!/bin/bash

sudo RUST_BACKTRACE=full $HOME/.cargo/bin/cargo run -- \
  --sandboxes 4 \
  --checker ./test/checker_fp.cpp \
  --language cpp17 \
  --metadata ./test/metadata.abc157d.yml \
  --source ./test/abc157d.cpp \
  --testcases ./test/abc157d \
  --testlib ./test/testlib.h \
  --verdict-format yaml \
  --verdict ./verdict.abc157d.yml \
  $* \
  1> >(sed $'s,.*,\e[1;33m&\e[m,'>&2);