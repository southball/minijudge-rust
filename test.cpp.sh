#!/bin/bash

sudo /home/jamiechoi/.cargo/bin/cargo run -- \
  --sandboxes 16 \
  --checker ./test/checker.cpp \
  --language cpp17 \
  --metadata ./test/metadata.cpp.yml \
  --source ./test/source.cpp \
  --testcases ./test/testcases \
  --testlib ./test/testlib.h \
  1> >(sed $'s,.*,\e[1;33m&\e[m,'>&2);