#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

cargo llvm-cov report --lcov | awk '
  /^SF:/ { file = substr($0, 4) }
  /^LF:/ { found = substr($0, 4) }
  /^LH:/ { hit = substr($0, 4) }
  /^end_of_record/ && file ~ /\/src\/domain\// {
    sub(ENVIRON["PWD"] "/", "", file)
    printf "%4d/%-4d lines  %s\n", hit, found, file
    if (hit < found) { short++ }
  }
  END {
    if (short > 0) {
      printf "\n%d file(s) in src/domain/ below 100%% line coverage\n", short
      exit 1
    }
  }
'
