#!/bin/bash

sudo $HOME/.cargo/bin/cargo run -- \
  --sandboxes 4 \
  --checker ./test/checker.cpp \
  --language cpp17 \
  --metadata ./test/metadata.cpp.yml \
  --source ./test/source.cpp \
  --testcases ./test/testcases \
  --testlib ./test/testlib.h \
  --verdict-format yaml \
  --verdict ./verdict.cpp.yml \
  $* \
  1> >(sed $'s,.*,\e[1;33m&\e[m,'>&2);