#!/bin/bash

sudo /home/jamiechoi/.cargo/bin/cargo run -- \
  --sandboxes 16 \
  --checker ./test/checker.cpp \
  --language python3 \
  --metadata ./test/metadata.py.yml \
  --source ./test/source.py \
  --testcases ./test/testcases \
  --testlib ./test/testlib.h \
  1> >(sed $'s,.*,\e[1;33m&\e[m,'>&2);