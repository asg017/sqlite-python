#!/bin/bash
hyperfine --warmup 10 \
'sqlite3x :memory: "select count(*) from generate_series(1, 1e6)"' \
  'sqlite3x :memory: ".load target/release/libpy0" "select count(*) from py_each(py_eval(\"range(1000000)\"))"'