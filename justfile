#!/usr/bin/env just --justfile

bench_vm:
  echo "cd ~/Desktop/Parallels\ Shared\ Folders/Home && cd CLionProjects/temple && cargo bench iai" | ssh parallels