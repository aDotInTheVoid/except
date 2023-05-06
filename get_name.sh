#!/bin/bash
set -eou pipefail

rfc="$2"

i=$(echo $2 | sd 'rfc(\d+)\.txt' '$1')

cat names.txt | rg "^$i " | sd '^\d{4}' ''